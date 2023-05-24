use std::error::Error;

use base64::{engine::general_purpose, Engine};
use derive_builder::Builder;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct PaypalClient {
  reqwest_client: reqwest::Client,
}

impl PaypalClient {
  pub fn new() -> Result<Self, Box<dyn Error>> {
    info!("Initializing Paypal client...");
    let client_id = std::env::var("CLIENT_ID").expect("CLIENT_ID must be set in .env file");
    let secret = std::env::var("SECRET").expect("SECRET must be set in .env file");
    let token = {
      let token = format! {"{}:{}", client_id, secret};
      let token = general_purpose::STANDARD.encode(token);
      format! {"Basic {}", token}
    };
    let mut headers = reqwest::header::HeaderMap::new();
    let mut authorizition = reqwest::header::HeaderValue::from_str(token.as_str()).unwrap();
    authorizition.set_sensitive(true);
    headers.insert(reqwest::header::AUTHORIZATION, authorizition);
    let client = reqwest::ClientBuilder::new()
      .default_headers(headers)
      .build()?;
    info!("Paypal client initialized");
    Ok(Self {
      reqwest_client: client,
    })
  }
}

fn gen_url(path: String) -> Result<String, Box<dyn Error>> {
  let is_sandbox =
    std::env::var("PAYPAL_ENV").expect("PAYPAL_ENV must be set in .env file") == "sandbox";
  let base_url = {
    if is_sandbox {
      "https://api-m.sandbox.paypal.com/v2"
    } else {
      "https://api-m.paypal.com/v2"
    }
  };
  Ok(format! {"{}{}", base_url, path})
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payer {
  address: Value,
  email_address: String,
  name: Value,
  pub payer_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PurchaseUnitPaymentsAuthorization {
  amount: Value,
  create_time: String,
  expiration_time: String,
  id: String,
  links: Vec<Value>,
  payer_id: Option<String>,
  seller_protection: Value,
  status: String,
  update_time: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerchaseUnitPayments {
  authorizations: Vec<PurchaseUnitPaymentsAuthorization>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PurchaseUnit {
  amount: Value,
  description: Option<String>,
  items: Vec<Value>,
  payee: Value,
  payments: PerchaseUnitPayments,
  reference_id: String,
  shipping: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payment {
  create_time: Option<String>,
  id: String,
  intent: String,
  links: Vec<Value>,
  pub payer: Payer,
  payment_source: Value,
  purchase_units: Vec<PurchaseUnit>,
  status: String,
  update_time: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Builder)]
#[builder(setter(into), default)]
pub struct BriefPayment {
  pub authorization_id: String,
  pub order_id: String,
  pub capture_id: Option<String>,
  pub payer_id: String,
  pub payer_email: String,
  pub create_time: Option<String>,
}

pub trait ToBriefPayment {
  fn to_brief_payment(&self) -> Result<BriefPayment, Box<dyn Error>>;
}

impl ToBriefPayment for Payment {
  fn to_brief_payment(&self) -> Result<BriefPayment, Box<dyn Error>> {
    Ok(
      BriefPaymentBuilder::default()
        .authorization_id(self.get_id()?)
        .order_id(self.id.clone())
        .payer_id(self.payer.payer_id.clone())
        .payer_email(self.payer.email_address.clone())
        .create_time(self.create_time.clone())
        .build()?,
    )
  }
}

impl BriefPayment {
  pub fn from(t: impl ToBriefPayment) -> Result<Self, Box<dyn Error>> {
    t.to_brief_payment()
  }
}

impl Payment {
  pub fn get_id(&self) -> Result<String, Box<dyn Error>> {
    let purchase_unit = self.purchase_units.get(0).ok_or("No purchase unit")?;
    let payment = purchase_unit
      .payments
      .authorizations
      .get(0)
      .ok_or("No payment")?;
    Ok(payment.id.clone())
  }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CapturedPayment {
  pub id: String,
  pub status: String,
  links: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Refund {
  pub id: String,
  pub links: Vec<Value>,
  pub status: String,
}

impl PaypalClient {
  async fn get_payment(&self, order_id: &String) -> Result<BriefPayment, Box<dyn Error>> {
    let client = &self.reqwest_client;
    let order_details: Payment = client
      .get(gen_url(format! {"/checkout/orders/{}", order_id})?)
      .send()
      .await?
      .json()
      .await?;
    BriefPayment::from(order_details)
  }
  async fn capture_payment(
    &self,
    authorization_id: String,
  ) -> Result<CapturedPayment, Box<dyn Error>> {
    let client = &self.reqwest_client;
    let url = gen_url(format! {"/payments/authorizations/{}/capture", authorization_id})?;
    let res = client
      .post(url)
      .json(&json!({"note_to_payer": "验证通过"}))
      .send()
      .await?
      .json()
      .await?;
    Ok(res)
  }
  async fn refund_payment(&self, capture_id: String) -> Result<Refund, Box<dyn Error>> {
    let client = &self.reqwest_client;
    let url = gen_url(format! {"/payments/captures/{}/refund", capture_id})?;
    let res = client
      .post(url)
      .json(&json!({"note_to_payer": "验证完成，进行退款"}))
      .send()
      .await?
      .json()
      .await?;
    Ok(res)
  }
}

impl BriefPayment {
  pub async fn from_order_id(
    order_id: &String,
    client: &PaypalClient,
  ) -> Result<Self, Box<dyn Error>> {
    client.get_payment(order_id).await
  }
  pub async fn capture(&mut self, client: &PaypalClient) -> Result<&mut Self, Box<dyn Error>> {
    let capture_res = client
      .capture_payment(self.authorization_id.clone())
      .await?;
    self.capture_id = Some(capture_res.id);
    Ok(self)
  }
  pub async fn refund(&mut self, client: &PaypalClient) -> Result<Refund, Box<dyn Error>> {
    client
      .refund_payment(self.capture_id.clone().unwrap())
      .await
  }
}
