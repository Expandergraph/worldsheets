use crate::{
    config::db::Pool,
    models::user::{User, UserDTO, LoginDTO},
    models::user_token::UserToken,
    utils::{constants, errors::ServiceError, token_utils},
};

use actix_web::{
    http::{
        StatusCode,
        header::HeaderValue,
    },
    web,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize)]
pub struct TokenBodyResponse{
    pub token: String,
    pub token_type: String,
}

pub fn signup(user: UserDTO, pool: &web::Data<Pool>) -> Result<String, ServiceError>{
    match User::signup(user, &pool.get().unwrap()){
        Ok(message) => Ok(message),
        Err(message) => Err(ServiceError::new(StatusCode::BAD_REQUEST, message)),
    }
}

pub fn login(login: LoginDTO, pool: &web::Data<Pool>) -> Result<TokenBodyResponse, ServiceError>{
    match User::login(login, &pool.get().unwrap()){
        Some(logged_user) => {
            match serde_json::from_value(json!({"token": UserToken::generate_token(&logged_user), "token_type":"bearer"})){
                Ok(token_res) => {
                    if logged_user.login_session.is_empty(){
                        Err(ServiceError::new(StatusCode::UNAUTHORIZED, constants::MESSAGE_LOGIN_FAILED.to_string()))
                    }else{
                        Ok(token_res)
                    }
                }
                Err(_) => Err(ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, constants::MESSAGE_INTERNAL_SERVER_ERROR.to_string()))
            }
        },
        None => Err(ServiceError::new(StatusCode::UNAUTHORIZED, constants::MESSAGE_USER_NOT_FOUND.to_string()))
    }
}

pub fn logout(authen_header: &HeaderValue, pool: &web::Data<Pool>) -> Result<(), ServiceError>{
    if let Ok(auth_str) = authen_header.to_str(){
        if auth_str.starts_with("bearer"){
            let token = auth_str[6..].trim();
            if let Ok(token_data) = token_utils::decode_token(token.to_string()){
                if let Ok(username) = token_utils::verify_token(&token_data, pool){
                    if let Ok(user) = User::find_user_by_username(&username, &pool.get().unwrap()){
                        User::logout(user.id, &pool.get().unwrap());
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, constants::MESSAGE_PROCESS_TOKEN_ERROR.to_string()))
}