use std::rc::Rc;

use crate::{
    method::diagnostic::{CfnLinter, Lint},
    model::{
        method::{diagnostic, initialise, NotificationMethod, RequestMethod},
        Error, ErrorCode, Message, Notification, Request, RequestId, Response,
    },
};

#[derive(Debug, Clone)]
enum State {
    Uninitialised,
    Initialised(initialise::Params),
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct MessageHandler {
    state: State,
    linter: Rc<dyn Lint>,
}

impl Default for MessageHandler {
    fn default() -> Self {
        Self {
            state: State::Uninitialised,
            linter: Rc::new(CfnLinter),
        }
    }
}

impl MessageHandler {
    pub fn handle(&mut self, message: Message) -> Option<Response> {
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

    fn handle_request_batch(&mut self, requests: Vec<Request>) -> Response {
        Response::Batch(
            requests
                .into_iter()
                .map(|request| self.handle_request(request))
                .collect(),
        )
    }

    fn handle_request(&mut self, request: Request) -> Response {
        match self.state {
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
        match self.state {
            State::Uninitialised | State::Shutdown => {
                if let NotificationMethod::Exit = notification.method() {
                    self.exit()
                }
            }
            State::Initialised(_) => todo!(),
        }
    }

    fn initialise(&mut self, id: &RequestId, params: &initialise::Params) -> Response {
        self.state = State::Initialised(params.clone());
        let result = initialise::Result::default();
        Response::success(id, crate::model::ResponseResult::Initialise(result))
    }

    fn shutdown(&mut self, id: &RequestId) -> Response {
        self.state = State::Shutdown;
        Response::success(id, crate::model::ResponseResult::Null)
    }

    fn text_document_diagnostic(&self, id: &RequestId, params: &diagnostic::Params) -> Response {
        let diagnostics = self.linter.lint(params);
        let result = diagnostic::Result::full("result", diagnostics);
        Response::success(
            id,
            crate::model::ResponseResult::TextDocumentDiagnostic(result),
        )
    }

    fn uninitialised_request(&self, id: &RequestId) -> Response {
        let error = Error::new(
            ErrorCode::ServerNotInitialised,
            "Server not initialised",
            None,
        );
        Response::error(id, error)
    }

    fn request_post_shutdown(&self, id: &RequestId) -> Response {
        let error = Error::new(ErrorCode::InvalidRequest, "Server has been shutdown", None);
        Response::error(id, error)
    }

    fn exit(&self) {
        std::process::exit(0);
    }
}
