use std::sync::{Arc, Mutex};

use crate::{
    method::diagnostic::{CfnLinter, Lint},
    model::{
        method::{diagnostic, initialise, NotificationMethod, RequestMethod},
        Error, ErrorType, Message, Notification, Request, RequestId, Response,
    },
};

#[derive(Debug, Clone)]
enum State {
    Uninitialised,
    Initialised(initialise::Params),
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct Server {
    state: Arc<Mutex<State>>,
    linter: Arc<dyn Lint + Send + Sync>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(State::Uninitialised)),
            linter: Arc::new(CfnLinter),
        }
    }
}

impl Server {
    pub fn handle(&self, message: Message) -> Option<Response> {
        tracing::info!("Receieved message: '{:?}'", message);
        match message {
            Message::Request(request) => Some(self.handle_request(request)),
            Message::BatchRequest(requests) => Some(self.handle_request_batch(requests)),
            Message::Notification(notification) => {
                self.handle_notification(notification);
                None
            }
        }
    }

    fn handle_request_batch(&self, requests: Vec<Request>) -> Response {
        Response::Batch(
            requests
                .into_iter()
                .map(|request| self.handle_request(request))
                .collect(),
        )
    }

    fn handle_request(&self, request: Request) -> Response {
        let state = {
            let lock = self.state.lock().expect("Can acquire lock");
            lock.clone()
        };
        match state {
            State::Uninitialised => match request.method() {
                RequestMethod::Initialise(params) => self.initialise(request.id(), params),
                _ => self.uninitialised_request(request.id()),
            },
            State::Shutdown => self.request_post_shutdown(request.id()),
            State::Initialised(_) => match request.method() {
                RequestMethod::Shutdown => self.shutdown(request.id()),
                RequestMethod::TextDocumentDiagnostic(params) => {
                    self.text_document_diagnostic(request.id(), params)
                }
                _ => todo!(),
            },
        }
    }

    fn handle_notification(&self, notification: Notification) {
        let state = {
            let lock = self.state.lock().expect("Can acquire lock");
            lock.clone()
        };
        match state {
            State::Uninitialised | State::Shutdown => {
                if let NotificationMethod::Exit = notification.method() {
                    self.exit()
                }
            }
            State::Initialised(_) => todo!(),
        }
    }

    fn initialise(&self, id: &RequestId, params: &initialise::Params) -> Response {
        let mut state = self.state.lock().expect("Can acquire lock");
        *state = State::Initialised(params.clone());
        let result = initialise::Result::default();

        match serde_json::to_value(result) {
            Ok(value) => Response::success(id, value),
            Err(_) => {
                let error = Error::new(
                    ErrorType::Internal.code(),
                    "Failed to serialize result",
                    Some(initialise::Error::default().to_value()),
                );
                Response::error(id, error)
            }
        }
    }

    fn shutdown(&self, id: &RequestId) -> Response {
        let mut state = self.state.lock().expect("Can acquire lock");
        *state = State::Shutdown;
        Response::success(id, serde_json::Value::Null)
    }

    fn text_document_diagnostic(&self, id: &RequestId, params: &diagnostic::Params) -> Response {
        let diagnostics = self.linter.lint(params);
        let result = serde_json::to_value(diagnostic::Result::full("result", diagnostics));
        match result {
            Ok(value) => Response::success(id, value),
            Err(_) => {
                let error = Error::new(
                    ErrorType::Internal.code(),
                    "Failed to serialize result",
                    None,
                );
                Response::error(id, error)
            }
        }
    }

    fn uninitialised_request(&self, id: &RequestId) -> Response {
        let error = Error::new(
            ErrorType::ServerNotInitialised.code(),
            "Server not initialised",
            None,
        );
        Response::error(id, error)
    }

    fn request_post_shutdown(&self, id: &RequestId) -> Response {
        let error = Error::new(
            ErrorType::InvalidRequest.code(),
            "Server has been shutdown",
            None,
        );
        Response::error(id, error)
    }

    fn exit(&self) {
        std::process::exit(0);
    }
}
