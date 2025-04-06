use std::{convert::Infallible, error::Error, net::SocketAddr, ops::Div, sync::{Arc, Mutex}, time::{Duration, Instant}};
use chrono::prelude::*;
use tokio::{net::TcpListener, task};
use std::thread;
use rppal::gpio::{Gpio, Trigger};
use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use serde::{Serialize, Deserialize};

// this amplifier type provides default trait functionality for Instant
#[derive(Clone, Copy)]
pub struct DefaultableInstant {
    pub instant : Instant
}
impl DefaultableInstant {
    pub fn now() -> Self { Self::default() }
}
impl Default for DefaultableInstant {
    fn default() -> Self {
        Self { instant : Instant::now() }
    }
}

// this type stores the state for the geiger counter.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct GeigerCounterState {
    #[serde(skip_serializing, skip_deserializing)]
    pub last_reset: DefaultableInstant,
    pub last_count: Option<DateTime<Utc>>,
    pub count: u64,
    pub last_reading_cpm: Option<f64>,
}
impl Default for GeigerCounterState {
    fn default() -> Self {
        Self {
            last_reset: DefaultableInstant::default(),
            last_count: None,
            last_reading_cpm: None,
            count: 0,
        }
    }
}

// this function handles the complex type setup for returning the cloned geiger state serialized to json
async fn read_counter(_: Request<Incoming>, state: &Arc<Mutex<GeigerCounterState>>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from(serde_json::to_string(&state.lock().unwrap().clone()).unwrap()))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // setup the initial state of the geiger counter.
    let state = Arc::new(Mutex::new(GeigerCounterState::default()));
    
    // create a new state reference for the pin interupt to be able to use.
    let state_pin_interrupt_handler = Arc::clone(&state);

    // setup the reset interval and and pin reference.
    let gpio_read_pin = 27;
    let gpio = Gpio::new().unwrap();
    let mut input_pin = gpio.get(gpio_read_pin).unwrap().into_input_pulldown();
    
    // setup the interupt handler for the pin.
    input_pin
        .set_async_interrupt(Trigger::RisingEdge, None, move |_event| {
            // setup a cloned reference to the state so that it will not consume the outer reference for the next interrupt
            let state_pin_interrupt_handler = Arc::clone(&state_pin_interrupt_handler);
            let mut locked_state = state_pin_interrupt_handler.lock().unwrap();

            // increment the counter once as there has been a beta or gamma class particle that has hit the gm tube.
            locked_state.count += 1;

            // the subtraction of 100 microseconds here is to compensate for the detection circuit propagation time. 
            locked_state.last_count = Some(Utc::now() - Duration::from_micros(100));
        })
        .unwrap();
    
    // setup the reset background thread.
    let state_reset_handler = Arc::clone(&state);
    let reset_interval_frequency = Duration::from_secs_f64(60.);
    let reset_thread_poll_frequency = Duration::from_secs_f64(1./60.);
    let _reset_thread = thread::spawn(move || {
        loop {
            thread::sleep(reset_thread_poll_frequency);

            let state_reset_handler = Arc::clone(&state_reset_handler);
            let mut locked_state = state_reset_handler.lock().unwrap();
        
            if locked_state.last_reset.instant.elapsed() >= reset_interval_frequency {
                locked_state.last_reading_cpm = (locked_state.count as f64)
                    .div(locked_state.last_reset.instant.elapsed().as_secs_f64().div(60.))
                    .into();

                locked_state.count = 0;
                locked_state.last_reset = DefaultableInstant::now();
            }
        }
    });
    
    // setup the ip address and tcp listener to handle http requests.
    let addr = SocketAddr::from(([0,0,0,0], 1986));
    let listener = TcpListener::bind(addr).await?;

    // setup a request handling loop that spawns a tokio task every time a request is made and handles it by reading the geiger counter state and returns a copy.
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state_request_handler = Arc::clone(&state);
        task::spawn(async move {
            // setup a cloned reference to the state so that it will not consume the outer reference for the next request.
            let state_request_handler = Arc::clone(&state_request_handler);
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|e| read_counter(e, &state_request_handler)))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
