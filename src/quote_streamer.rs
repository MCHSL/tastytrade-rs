use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::sync::Mutex;
use std::{ffi::CString, fmt::Display};

use dxfeed::{dxf_event_flags_t, Event};
use widestring::WideCString;

use crate::api::order::AsSymbol;
use crate::Result;
use crate::TastyTrade;

const SUCCESS: i32 = dxfeed::DXF_SUCCESS as i32;

#[derive(Debug, thiserror::Error)]
pub enum DxFeedError {
    CreateConnectionError,
}

impl Display for DxFeedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DxFeed error: {:?}", self)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct SubscriptionId(usize);

#[derive(Debug)]
pub struct QuoteSubscription {
    pub id: SubscriptionId,
    subscription: dxfeed::dxf_subscription_t,
    event_receiver: flume::Receiver<dxfeed::Event>,
    sender_ptr: *mut flume::Sender<dxfeed::Event>,
}

impl Drop for QuoteSubscription {
    fn drop(&mut self) {
        if !self.subscription.is_null() {
            assert_eq!(SUCCESS, unsafe {
                dxfeed::dxf_close_subscription(self.subscription)
            });
            self.subscription = std::ptr::null_mut();
            unsafe {
                std::mem::drop(Box::from_raw(self.sender_ptr));
            }
        }
    }
}

impl QuoteSubscription {
    pub fn add_symbols<S: AsSymbol>(&self, symbols: &[S]) {
        let symbols: Vec<WideCString> = symbols
            .iter()
            .map(|sym| WideCString::from_str(sym.as_symbol().0).unwrap())
            .collect();

        let mut symbol_pointers: Vec<*const i32> = symbols
            .iter()
            .map(|sym| sym.as_ptr() as *const i32)
            .collect();

        let c_syms: *mut dxfeed::dxf_const_string_t =
            symbol_pointers.as_mut_slice().as_ptr() as *mut dxfeed::dxf_const_string_t;

        assert_eq!(SUCCESS, unsafe {
            dxfeed::dxf_add_symbols(self.subscription, c_syms, symbols.len() as i32)
        });
    }

    pub async fn get_event(&self) -> std::result::Result<Event, flume::RecvError> {
        self.event_receiver.recv_async().await
    }

    pub extern "C" fn sub_callback(
        event_type: std::os::raw::c_int,
        sym: dxfeed::dxf_const_string_t,
        data: *const dxfeed::dxf_event_data_t,
        _data_count: i32, // always 1, and deprecated
        sender_ptr: *mut std::ffi::c_void,
    ) {
        let sender = unsafe { &mut *(sender_ptr as *mut _ as *mut flume::Sender<dxfeed::Event>) };
        match dxfeed::Event::try_from_c(event_type, sym, data) {
            Ok(evt) => _ = sender.send(evt),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

#[derive(Debug)]
pub struct QuoteStreamer {
    host: String,
    token: String,

    connection: dxfeed::dxf_connection_t,
    subscriptions: HashMap<SubscriptionId, QuoteSubscription>,
    next_sub_id: usize,
}

unsafe impl Send for QuoteStreamer {}
// unsafe impl Sync for QuoteStreamer {}

impl QuoteStreamer {
    pub async fn connect(tasty: &TastyTrade) -> Result<QuoteStreamer> {
        let tokens = tasty.quote_streamer_tokens().await?;

        let mut this = Self {
            host: tokens.streamer_url,
            token: tokens.token,

            connection: std::ptr::null_mut(),
            subscriptions: HashMap::new(),
            next_sub_id: 0,
        };

        let c_host = CString::new(this.host.clone()).unwrap();
        let c_token = CString::new(this.token.clone()).unwrap();

        assert_eq!(SUCCESS, unsafe {
            dxfeed::dxf_create_connection_auth_bearer(
                c_host.as_ptr(), // const char* address,
                c_token.as_ptr(),
                Some(Self::termination_listener), // dxf_conn_termination_notifier_t notifier,
                Some(Self::sub_listener), // dxf_conn_status_notifier_t conn_status_notifier,
                None,                     // dxf_socket_thread_creation_notifier_t stcn,
                None,                     // dxf_socket_thread_destruction_notifier_t stdn,
                std::ptr::null_mut(),     // void* user_data,
                &mut this.connection,     // OUT dxf_connection_t* connection);
            )
        });

        Ok(this)
    }

    extern "C" fn termination_listener(
        _connection: dxfeed::dxf_connection_t,
        _user_data: *mut ::std::os::raw::c_void,
    ) {
        //eprintln!("!!! conn terminated !!!");
    }

    extern "C" fn sub_listener(
        _connection: dxfeed::dxf_connection_t,
        _old_status: dxfeed::dxf_connection_status_t,
        _new_status: dxfeed::dxf_connection_status_t,
        _sender_ptr: *mut std::ffi::c_void,
    ) {
        //eprintln!("!!! sub !!! {} => {}", old_status, new_status);
    }

    pub fn create_sub(&mut self, flags: i32) -> &QuoteSubscription {
        let mut subscription: dxfeed::dxf_subscription_t = std::ptr::null_mut();
        let (event_sender, event_receiver) = flume::unbounded();
        let event_sender = Box::new(event_sender);
        let sender_ptr = Box::into_raw(event_sender);
        assert_eq!(SUCCESS, unsafe {
            dxfeed::dxf_create_subscription(self.connection, flags, &mut subscription)
        });
        assert_eq!(SUCCESS, unsafe {
            dxfeed::dxf_attach_event_listener(
                subscription,
                Some(QuoteSubscription::sub_callback),
                sender_ptr as *mut std::ffi::c_void,
            )
        });

        let id = SubscriptionId(self.next_sub_id);
        self.next_sub_id += 1;

        self.subscriptions.insert(
            id,
            QuoteSubscription {
                id,
                subscription,
                event_receiver,
                sender_ptr,
            },
        );

        self.get_sub(id).unwrap()
    }

    pub fn get_sub(&self, id: SubscriptionId) -> Option<&QuoteSubscription> {
        self.subscriptions.get(&id)
    }

    // pub fn subscribe<S: AsSymbol>(&self, symbols: &[S]) {
    //     // let sender_ptr: *mut std::ffi::c_void =
    //     //     &self.event_sender as *const flume::Sender<dxfeed::Event> as *mut std::ffi::c_void;
    //     // let mut sub = *self.subscription.lock().unwrap();
    //     let conn = *self.connection.lock().unwrap();

    //     let symbols: Vec<WideCString> = symbols
    //         .iter()
    //         .map(|sym| WideCString::from_str(sym.as_symbol().0).unwrap())
    //         .collect();

    //     let mut symbol_pointers: Vec<*const i32> = symbols
    //         .iter()
    //         .map(|sym| sym.as_ptr() as *const i32)
    //         .collect();

    //     let c_syms: *mut dxfeed::dxf_const_string_t =
    //         symbol_pointers.as_mut_slice().as_ptr() as *mut dxfeed::dxf_const_string_t;

    //     if sub.is_null() {
    //         assert_eq!(SUCCESS, unsafe {
    //             dxfeed::dxf_create_subscription(
    //                 conn,
    //                 dxfeed::DXF_ET_QUOTE | dxfeed::DXF_ET_GREEKS,
    //                 &mut sub,
    //             )
    //         });
    //     }

    //     assert_eq!(SUCCESS, unsafe {
    //         dxfeed::dxf_attach_event_listener(sub, Some(Self::evt_listener), sender_ptr)
    //     });

    //     assert_eq!(SUCCESS, unsafe {
    //         dxfeed::dxf_add_symbols(sub, c_syms, symbols.len() as i32)
    //     });
    // }

    // pub async fn handle_events<F>(&self, f: F)
    // where
    //     F: Fn(Event),
    // {
    //     while let Ok(ev) = self.event_receiver.recv_async().await {
    //         f(ev);
    //     }
    // }

    // pub async fn get_event(&self) -> std::result::Result<Event, flume::RecvError> {
    //     self.event_receiver.recv_async().await
    // }
}

impl Drop for QuoteStreamer {
    fn drop(&mut self) {
        // let mut sub = self.subscription.lock().unwrap();
        self.subscriptions.clear();
        // if !sub.is_null() {
        //     assert_eq!(SUCCESS, unsafe { dxfeed::dxf_close_subscription(*sub) });
        //     *sub = std::ptr::null_mut();
        // }
        if !self.connection.is_null() {
            unsafe { dxfeed::dxf_close_connection(self.connection) };
            self.connection = std::ptr::null_mut();
        }
    }
}

impl TastyTrade {
    pub async fn create_quote_streamer(&self) -> Result<QuoteStreamer> {
        QuoteStreamer::connect(self).await
    }
}
