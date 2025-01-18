use crate::{
    method::diagnostic::{CfnLinter, Lint},
    model::{
        method::{
            diagnostic,
            initialise::{self, ClientInfo},
            NotificationMethod, RequestMethod,
        },
        Error, ErrorCode, ErrorResponse, Message, Notification, Request, RequestId, Response,
        ResponseResult, SuccessResponse,
    },
};

#[derive(Debug, Clone)]
enum State {
    Uninitialised,
    Initialised(initialise::Params),
    Shutdown,
}

#[derive(Debug)]
pub struct MessageHandler {
    client_process_id: Option<String>,
    state: State,
    linter: Box<dyn Lint>,
}

impl MessageHandler {
    pub fn new(client_process_id: Option<&String>) -> Self {
        Self {
            client_process_id: client_process_id.cloned(),
            state: State::Uninitialised,
            linter: Box::new(CfnLinter),
        }
    }

    pub fn handle(&mut self, message: Message) -> Option<Response> {
        match message {
            Message::Request(request) => Some(self.handle_request(&request)),
            Message::BatchRequest(requests) => Some(self.handle_request_batch(requests)),
            Message::Notification(notification) => {
                self.handle_notification(&notification);
                None
            }
        }
    }

    fn handle_request_batch(&mut self, requests: Vec<Request>) -> Response {
        Response::Batch(
            requests
                .into_iter()
                .map(|request| self.handle_request(&request))
                .collect(),
        )
    }

    fn handle_request(&mut self, request: &Request) -> Response {
        match self.state {
            State::Uninitialised => match request.method() {
                RequestMethod::Initialise(params) => self.initialise(request.id(), params),
                _ => uninitialised_request(request.id()),
            },
            State::Shutdown => request_post_shutdown(request.id()),
            State::Initialised(_) => match request.method() {
                RequestMethod::Shutdown => self.shutdown(request.id()),
                RequestMethod::TextDocumentDiagnostic(params) => {
                    self.text_document_diagnostic(request.id(), params)
                }
                RequestMethod::Initialise(_) => already_initialised(request.id()),
            },
        }
    }

    fn handle_notification(&self, notification: &Notification) {
        match self.state {
            State::Uninitialised | State::Shutdown => {
                if let NotificationMethod::Exit = notification.method() {
                    MessageHandler::exit();
                }
            }
            State::Initialised(_) => {}
        }
    }

    fn initialise(&mut self, id: &RequestId, params: &initialise::Params) -> Response {
        tracing::info!(
            id = tracing::field::display(id),
            "Initialising server for client '{}'",
            params.client_info().unwrap_or(&ClientInfo::default())
        );
        self.state = State::Initialised(params.clone());
        let result = initialise::Result::default();
        let success = SuccessResponse::new(id, ResponseResult::Initialise(result));
        Response::Success(success)
    }

    fn shutdown(&mut self, id: &RequestId) -> Response {
        tracing::info!(id = tracing::field::display(id), "Shutting down server");
        self.state = State::Shutdown;
        let success = SuccessResponse::new(id, ResponseResult::Null);
        Response::Success(success)
    }

    fn text_document_diagnostic(&self, id: &RequestId, params: &diagnostic::Params) -> Response {
        tracing::debug!(
            id = tracing::field::display(id),
            "Generating diagnostics for file '{}'",
            params.uri()
        );
        match self.linter.lint(params) {
            Ok(diagnostics) => {
                let result = diagnostic::Result::full("result", diagnostics);
                let success =
                    SuccessResponse::new(id, ResponseResult::TextDocumentDiagnostic(result));
                Response::Success(success)
            }
            Err(error) => {
                tracing::error!(
                    id = tracing::field::display(id),
                    "Failed to generate diagnostics: {error}"
                );
                let error = Error::new(ErrorCode::Internal, "Failed to generate diagnostics", None);
                Response::Error(ErrorResponse::new(id, error))
            }
        }
    }

    fn exit() {
        tracing::info!("Received exit notification. Exiting...");
        std::process::exit(0);
    }
}

fn uninitialised_request(id: &RequestId) -> Response {
    let error = Error::new(
        ErrorCode::ServerNotInitialised,
        "Server not initialised",
        None,
    );
    Response::Error(ErrorResponse::new(id, error))
}

fn already_initialised(id: &RequestId) -> Response {
    let error = Error::new(
        ErrorCode::ServerAlreadyInitialised,
        "Server already initialised",
        None,
    );
    Response::Error(ErrorResponse::new(id, error))
}

fn request_post_shutdown(id: &RequestId) -> Response {
    let error = Error::new(ErrorCode::InvalidRequest, "Server has been shutdown", None);
    Response::Error(ErrorResponse::new(id, error))
}
