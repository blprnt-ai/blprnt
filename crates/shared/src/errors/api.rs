#[derive(Debug, thiserror::Error)]
pub enum ApiError {
  #[error("failed to get user: {0}")]
  FailedToGetUser(String),

  #[error("failed to sign in: {0}")]
  FailedToSignIn(String),

  #[error("failed to sign out: {0}")]
  FailedToSignOut(String),

  #[error("failed to initialize user: {0}")]
  FailedToInitializeUser(String),

  #[error("failed to get models: {0}")]
  FailedToGetModels(String),

  #[error("failed to create payment intent: {0}")]
  FailedToCreatePaymentIntent(String),

  #[error("failed to create checkout session: {0}")]
  FailedToCreateCheckoutSession(String),

  #[error("failed to create billing portal session: {0}")]
  FailedToCreateBillingPortal(String),

  #[error("failed to list invoices: {0}")]
  FailedToListInvoices(String),

  #[error("failed to list payment methods: {0}")]
  FailedToListPaymentMethods(String),

  #[error("failed to get credit balance: {0}")]
  FailedToGetCreditBalance(String),

  #[error("failed to get blprnt response: {0}")]
  FailedToGetResponse(String),

  #[error("failed to open store: {0}")]
  FailedToOpenStore(String),
}
