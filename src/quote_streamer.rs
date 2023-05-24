use std::io::Write;
use std::pin::Pin;
use std::{ffi::CString, fmt::Display};

use dxfeed::Event;
use widestring::WideCString;

use crate::api::order::AsSymbol;
use crate::api::order::Symbol;
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

pub struct QuoteStreamer {
    host: String,
    token: String,

    connection: dxfeed::dxf_connection_t,
    subscription: dxfeed::dxf_subscription_t,

    event_receiver: flume::Receiver<dxfeed::Event>,
    event_sender: flume::Sender<dxfeed::Event>,
}

unsafe impl Send for QuoteStreamer {}

impl QuoteStreamer {
    pub fn connect<H: AsRef<str>, T: AsRef<str>>(host: H, token: T) -> Result<Self> {
        let (sender, receiver) = flume::unbounded();

        let mut this = Self {
            host: host.as_ref().to_owned(),
            token: token.as_ref().to_owned(),

            connection: std::ptr::null_mut(),
            subscription: std::ptr::null_mut(),

            event_receiver: receiver,
            event_sender: sender,
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
        eprintln!("!!! conn terminated !!!");
    }

    extern "C" fn sub_listener(
        _connection: dxfeed::dxf_connection_t,
        old_status: dxfeed::dxf_connection_status_t,
        new_status: dxfeed::dxf_connection_status_t,
        _sender_ptr: *mut std::ffi::c_void,
    ) {
        eprintln!("!!! sub !!! {} => {}", old_status, new_status);
    }

    pub extern "C" fn evt_listener(
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

    pub fn subscribe<S: AsSymbol>(&mut self, symbols: &[S]) {
        let sender_ptr: *mut std::ffi::c_void =
            &mut self.event_sender as *mut _ as *mut std::ffi::c_void;

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
            dxfeed::dxf_create_subscription(
                self.connection,
                dxfeed::DXF_ET_QUOTE,
                &mut self.subscription,
            )
        });

        assert_eq!(SUCCESS, unsafe {
            dxfeed::dxf_attach_event_listener(
                self.subscription,
                Some(Self::evt_listener),
                sender_ptr,
            )
        });

        assert_eq!(SUCCESS, unsafe {
            dxfeed::dxf_add_symbols(self.subscription, c_syms, symbols.len() as i32)
        });
    }

    pub async fn handle_events<F>(&mut self, f: F)
    where
        F: Fn(Event),
    {
        while let Ok(ev) = self.event_receiver.recv_async().await {
            f(ev);
        }
    }
}

impl TastyTrade {
    pub async fn create_quote_streamer(&self) -> Result<Pin<Box<QuoteStreamer>>> {
        let tokens = self.quote_streamer_tokens().await?;
        let streamer = QuoteStreamer::connect(tokens.streamer_url, tokens.token)?;
        Ok(Box::pin(streamer))
    }
}
