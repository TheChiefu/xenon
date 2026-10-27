use std::collections::HashMap;

use axum::extract::{Query, State};
use serde::Deserialize;
use sqlx::{SqlitePool};

use crate::error::{AppError, Result};

#[derive(Debug, Deserialize)]
pub struct RedirectQuery {
    #[serde(flatten)]
    params: HashMap<String, String>
}

pub async fn redirect(
    State(pool): State<SqlitePool>,
    Query(query): Query<RedirectQuery>,
) -> Result<()> {

    tracing::trace!("here");
    println!("here");
    println!("{:?}", query);

    match query.params.get("openid.claimed_id") {
        Some(text) =>  println!("{}", text),
        None => println!("Nothing recieved")
    };
   

    // Read steam redirect response from GET user's browser makes to this endpoint with openID details
    
    // Parse out user details

    // Redirect user to success page

    // Save user steam ID to DB

    Ok(())
}


/*
https://chat.eternus.app//auth/steam/callback
?openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0
&openid.mode=id_res
&openid.op_endpoint=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Flogin
&openid.claimed_id=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198031611156
&openid.identity=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198031611156
&openid.return_to=https%3A%2F%2Fchat.eternus.app%2F%2Fauth%2Fsteam%2Fcallback
&openid.response_nonce=2026-10-27T00%3A39%3A27Z1ptNfT%2BVZvpGbXcORLnw%2Bq7VUhI%3D
&openid.assoc_handle=1234567890
&openid.signed=signed,op_endpoint,claimed_id,identity,return_to,response_nonce,assoc_handle
&openid.sig=IwJvAlUwKaiuPGriofbgmzY572k%3D


https://chat.eternus.app//auth/steam/callback
?openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0
&openid.mode=id_res
&openid.op_endpoint=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Flogin
&openid.claimed_id=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198031611156
&openid.identity=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198031611156
&openid.return_to=https%3A%2F%2Fchat.eternus.app%2F%2Fauth%2Fsteam%2Fcallback
&openid.response_nonce=2026-10-27T00%3A39%3A27Z1ptNfT%2BVZvpGbXcORLnw%2Bq7VUhI%3D
&openid.assoc_handle=1234567890
&openid.signed=signed%2Cop_endpoint%2Cclaimed_id%2Cidentity%2Creturn_to%2Cresponse_nonce%2Cassoc_handle
&openid.sig=IwJvAlUwKaiuPGriofbgmzY572k%3D
*/