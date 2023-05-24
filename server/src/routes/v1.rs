use actix_web::{
  post,
  web::{scope, Data, Json},
  HttpResponse, Responder, Scope,
};
use log::info;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use crate::{
  api::BriefPayment,
  entities::record::ActiveModel as RecordModel,
  entities::{prelude::*, record::Column},
  AppState,
};

pub fn v1() -> Scope {
  scope("/v1").service(order_complete)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OrderCompleteRequest {
  order_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OrderCompleteResponse {
  message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  payer_id: Option<String>,
}

/// 文档“结构”一节中 1. 所述接口
#[post("/orderComplete")]
async fn order_complete(data: Data<AppState>, req: Json<OrderCompleteRequest>) -> impl Responder {
  let client = &data.paypal_client;
  let db = &data.conn;
  let mut payment = match BriefPayment::from_order_id(&req.order_id, client).await {
    Err(_) => {
      return HttpResponse::BadRequest().json("Invalid order id");
    }
    Ok(o) => o,
  };
  info!("{}", to_string_pretty(&payment).unwrap());
  let captured_payment = match payment.capture(client).await {
    Err(_) => {
      return HttpResponse::InternalServerError().json("Failed to capture payment");
    }
    Ok(o) => o,
  };
  let record = match Record::find()
    .filter(Column::PayerId.eq(&captured_payment.payer_id))
    .one(db)
    .await
  {
    Err(_) => {
      return HttpResponse::InternalServerError().json("Failed to access db");
    }
    Ok(o) => o,
  };
  let signed = record.is_some();
  info!("{}", to_string_pretty(&captured_payment).unwrap());
  let refund = captured_payment.refund(client).await;
  if refund.is_err() {
    return HttpResponse::InternalServerError().json("Failed to refund payment");
  };
  info!("{}", to_string_pretty(&refund.unwrap()).unwrap());
  if signed {
    let record = record.unwrap();
    if record.used {
      return HttpResponse::Conflict().json(OrderCompleteResponse {
        message: "You have already signed up".to_owned(),
        payer_id: None,
      });
    }
    return HttpResponse::Conflict().json(OrderCompleteResponse {
      message: "You have already signed up".to_owned(),
      payer_id: Some(record.payer_id),
    });
  }
  let res = RecordModel {
    order_id: Set(captured_payment.order_id.clone()),
    authorization_id: Set(captured_payment.authorization_id.clone()),
    capture_id: Set(captured_payment.capture_id.clone()),
    create_time: Set(captured_payment.create_time.clone()),
    payer_email: Set(captured_payment.payer_email.clone()),
    payer_id: Set(captured_payment.payer_id.clone()),
    ..Default::default()
  }
  .insert(db)
  .await;
  if res.is_err() {
    return HttpResponse::InternalServerError().json("Failed to insert payment");
  };
  HttpResponse::Ok().json(OrderCompleteResponse {
    message: "Success".to_owned(),
    payer_id: Some(captured_payment.payer_id.clone()),
  })
}
