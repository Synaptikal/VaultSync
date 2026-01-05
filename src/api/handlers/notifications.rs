//! Notification API handlers
//!
//! Handles email receipts, customer notifications, and trade-in quote emails.

use crate::api::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct EmailReceiptRequest {
    email: String,
}

pub async fn email_receipt(
    State(state): State<AppState>,
    Path(transaction_uuid): Path<Uuid>,
    Json(payload): Json<EmailReceiptRequest>,
) -> impl IntoResponse {
    // 1. Generate Receipt HTML
    let html_content = match state.system.receipts.generate_html(transaction_uuid).await {
        Ok(html) => html,
        Err(e) => {
            return (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response()
        }
    };

    // 2. Create Message
    let message = crate::services::notification::EmailMessage {
        to: payload.email,
        subject: format!("Receipt for Transaction {}", transaction_uuid),
        body: html_content,
        attachment_path: None,
    };

    // 3. Send Email
    match state.system.email.send_email(&message).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Receipt sent"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct NotificationRequest {
    message: String,
    channel: String, // "sms", "email", "both"
}

pub async fn notify_customer(
    State(state): State<AppState>,
    Path(customer_uuid): Path<Uuid>,
    Json(payload): Json<NotificationRequest>,
) -> impl IntoResponse {
    // 1. Fetch Customer
    let customer = match state.db.customers.get_by_id(customer_uuid).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Customer not found"})),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    let mut success_count = 0;
    let mut errors = Vec::new();

    // 2. Send SMS
    if payload.channel == "sms" || payload.channel == "both" {
        if let Some(phone) = &customer.phone {
            if !phone.is_empty() {
                match state.system.sms.send_sms(phone, &payload.message).await {
                    Ok(_) => success_count += 1,
                    Err(e) => errors.push(format!("SMS failed: {}", e)),
                }
            } else {
                errors.push("SMS requested but customer has no phone".to_string());
            }
        }
    }

    // 3. Send Email
    if payload.channel == "email" || payload.channel == "both" {
        if let Some(email) = &customer.email {
            if !email.is_empty() {
                let msg = crate::services::notification::EmailMessage {
                    to: email.clone(),
                    subject: "Notification from VaultSync".to_string(),
                    body: payload.message.clone(),
                    attachment_path: None,
                };
                match state.system.email.send_email(&msg).await {
                    Ok(_) => success_count += 1,
                    Err(e) => errors.push(format!("Email failed: {}", e)),
                }
            } else {
                errors.push("Email requested but customer has no email".to_string());
            }
        }
    }
    if success_count > 0 {
        (
            StatusCode::OK,
            Json(json!({
                "message": "Notification process completed",
                "sent": success_count,
                "errors": errors
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Failed to send notification", "details": errors})),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
pub struct EmailQuoteRequest {
    pub customer_uuid: Uuid,
    pub items: Vec<crate::buylist::BuylistItem>,
}

pub async fn email_trade_in_quote(
    State(state): State<AppState>,
    Json(payload): Json<EmailQuoteRequest>,
) -> impl IntoResponse {
    // 1. Fetch Customer
    let customer = match state.db.customers.get_by_id(payload.customer_uuid).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Customer not found"})),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    let email = match customer.email {
        Some(e) if !e.is_empty() => e,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Customer has no email address"})),
            )
                .into_response()
        }
    };

    // 2. Calculate Quotes & Build HTML
    let mut total_cash = 0.0;
    let mut total_credit = 0.0;
    let mut items_html = String::new();

    for item in payload.items {
        // Fetch Product for name
        let product_name = match state.commerce.product.get_by_id(item.product_uuid).await {
            Ok(Some(p)) => p.name,
            _ => "Unknown Product".to_string(),
        };

        // Calculate Quote
        match state
            .commerce
            .buylist
            .calculate_instant_quote(item.product_uuid, item.condition.clone())
            .await
        {
            Ok(quote) => {
                let line_cash = quote.cash_price * item.quantity as f64;
                let line_credit = quote.credit_price * item.quantity as f64;

                total_cash += line_cash;
                total_credit += line_credit;

                items_html.push_str(&format!(
                    "<tr>
                        <td>{}</td>
                        <td>{:?}</td>
                        <td>{}</td>
                        <td align='right'>${:.2}</td>
                        <td align='right'>${:.2}</td>
                    </tr>",
                    product_name, item.condition, item.quantity, line_cash, line_credit
                ));
            }
            Err(_) => {
                items_html.push_str(&format!(
                    "<tr>
                        <td>{} (Error calculating quote)</td>
                        <td>{:?}</td>
                        <td>{}</td>
                        <td align='right'>-</td>
                        <td align='right'>-</td>
                    </tr>",
                    product_name, item.condition, item.quantity
                ));
            }
        }
    }

    let html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <style>
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 8px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h2>Trade-In Quote for {}</h2>
    <p>Here is the quote you requested:</p>
    <table>
        <thead>
            <tr>
                <th>Product</th>
                <th>Condition</th>
                <th>Qty</th>
                <th align='right'>Cash Value</th>
                <th align='right'>Credit Value</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
        <tfoot>
            <tr style="font-weight: bold;">
                <td colspan="3" align="right">Total:</td>
                <td align="right">${:.2}</td>
                <td align="right">${:.2}</td>
            </tr>
        </tfoot>
    </table>
    <br>
    <p><em>Prices are subject to change and final inspection. Quote valid for 24 hours.</em></p>
</body>
</html>"#,
        customer.name, items_html, total_cash, total_credit
    );

    // 3. Send Email
    let msg = crate::services::notification::EmailMessage {
        to: email,
        subject: "Your Trade-In Quote from VaultSync".to_string(),
        body: html_body,
        attachment_path: None,
    };

    match state.system.email.send_email(&msg).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "Quote sent", "cash_total": total_cash, "credit_total": total_credit}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
