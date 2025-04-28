use super::error::ErrorCode;
use super::requests::Request;
use super::POOL_CONFIG;
use indy_vdr::common::error::VdrResult;
use indy_vdr::pool::{
    PoolBuilder, PoolRunner, PoolTransactions, RequestMethod, RequestResult, RequestResultMeta
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};

pub struct Pool {
    pool: RwLock<Option<PoolRunner>>,
    init_txns: PoolTransactions,
    node_weights: Option<NodeWeights>
}

impl Pool{
    fn new(pool: RwLock<Option<PoolRunner>>, init_txns: PoolTransactions, node_weights: Option<NodeWeights>) -> Pool{
        Pool {
            pool,
            init_txns,
            node_weights
        }
    }
}


pub type NodeWeights = HashMap<String, f32>;

#[uniffi::export]
fn open_pool(
    transactions_path: Option<String>,
    transactions: Option<String>,
    node_weights: Option<NodeWeights>,
) -> Result<Arc<Pool>, ErrorCode> {
    let txns = if let Some(txns) = transactions {
        PoolTransactions::from_json(txns.as_str())?
    } else if let Some(path) = transactions_path {
        PoolTransactions::from_json_file(path.as_str())?
    } else {
        return Err(ErrorCode::Input {
            error_message:
                "Invalid pool create parameters: must provide transactions or transactions_path"
                    .to_string(),
        });
    };

    let gcfg = read_lock!(POOL_CONFIG)?;
    let builder = PoolBuilder::new(gcfg.clone(), txns.clone()).node_weights(node_weights.clone());
    let pool = builder.into_runner(None)?;
    Ok(Arc::new(Pool::new(RwLock::new(Some(pool)), txns, node_weights)))
}

fn handle_request_result(
    result: VdrResult<(RequestResult<String>, RequestResultMeta)>,
) -> (ErrorCode, String) {
    match result {
        Ok((reply, _timing)) => match reply {
            RequestResult::Reply(body) => (ErrorCode::Success {}, body),
            RequestResult::Failed(err) => {
                let code = ErrorCode::from(err);
                (code, String::new())
            }
        },
        Err(err) => {
            let code = ErrorCode::from(err);
            (code, String::new())
        }
    }
}

async fn handle_pool_refresh(
    init_txns: PoolTransactions,
    new_txns: Option<PoolTransactions>,
    node_weights: Option<NodeWeights>
) -> Result<Option<PoolRunner>, ErrorCode> {
    let txns = match new_txns{
        Some(new_txns) => {
            let mut txn = init_txns.clone();
            txn.extend(new_txns.iter().map(|x| x.clone()));
            txn
        }
        None => init_txns,
    };
    let gcfg = read_lock!(POOL_CONFIG)?;
    let runner = PoolBuilder::new(gcfg.clone(), txns).node_weights(node_weights).refreshed(true).into_runner(None)?;
    Ok(Some(runner))
}

#[uniffi::export(async_runtime = "tokio")]
impl Pool {
    pub async fn refresh(&self) -> Result<(), ErrorCode> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (tx, rx) = oneshot::channel();
        let init_txns = self.init_txns.clone();
        let node_weights = self.node_weights.clone();
        read_pool!(self.pool)?.refresh(Box::new(move |result| {
            match result {
                Ok((new_txns, _metadata)) => {
                    let result = rt.block_on(handle_pool_refresh(init_txns, new_txns, node_weights));
                    let _ = tx.send(result);
                }
                Err(err) => {
                    let code = ErrorCode::from(err);
                    let _ = tx.send(Err(code));
                }
            };
        }))?;
        let result = rx.await.map_err(|err| ErrorCode::Unexpected {
            error_message: format!("Channel error: {}", err),
        })?;
        match result {
            Ok(runner) => {
                if let Some(runner) = runner {
                    *self.pool.write().await = Some(runner);
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_status(&self) -> Result<String, ErrorCode> {
        let (tx, rx) = oneshot::channel();
        read_pool!(self.pool)?.get_status(Box::new(move |result| {
            let (errcode, reply) = match result {
                Ok(status) => {
                    let status = status.serialize().unwrap();
                    (ErrorCode::Success {}, status)
                }
                Err(err) => {
                    let code = ErrorCode::from(err);
                    (code, String::new())
                }
            };
            let _ = tx.send((errcode, reply));
        }))?;
        let (errcode, reply) = rx.await.map_err(|err| ErrorCode::Unexpected {
            error_message: format!("Channel error: {}", err),
        })?;
        if errcode != (ErrorCode::Success {}) {
            return Err(errcode);
        }
        Ok(reply)
    }

    pub async fn get_transactions(&self) -> Result<String, ErrorCode> {
        let (tx, rx) = oneshot::channel();
        read_pool!(self.pool)?.get_transactions(Box::new(move |result| {
            let (errcode, reply) = match result {
                Ok(txns) => (ErrorCode::Success {}, txns.join("\n")),
                Err(err) => {
                    let code = ErrorCode::from(err);
                    (code, String::new())
                }
            };
            let _ = tx.send((errcode, reply));
        }))?;
        let (errcode, reply) = rx.await.map_err(|err| ErrorCode::Unexpected {
            error_message: format!("Channel error: {}", err),
        })?;
        if errcode != (ErrorCode::Success {}) {
            return Err(errcode);
        }
        Ok(reply)
    }

    pub async fn submit_action(
        &self,
        request: Arc<Request>,
        node_aliases: Option<Vec<String>>,
        timeout: Option<i64>,
    ) -> Result<String, ErrorCode> {
        request.set_method(RequestMethod::Full {
            node_aliases,
            timeout,
        })?;
        let req = take_req!(request.req)?;
        let (tx, rx) = oneshot::channel();
        read_pool!(self.pool)?.send_request(
            req,
            Box::new(move |result| {
                let (errcode, reply) = handle_request_result(result);
                let _ = tx.send((errcode, reply));
            }),
        )?;
        let (errcode, reply) = rx.await.map_err(|err| ErrorCode::Unexpected {
            error_message: format!("Channel error: {}", err),
        })?;
        if errcode != (ErrorCode::Success {}) {
            return Err(errcode);
        }
        Ok(reply)
    }

    pub async fn submit_request(&self, request: Arc<Request>) -> Result<String, ErrorCode> {
        let req = take_req!(request.req)?;
        let (tx, rx) = oneshot::channel();
        read_pool!(self.pool)?.send_request(
            req,
            Box::new(move |result| {
                let (errcode, reply) = handle_request_result(result);
                let _ = tx.send((errcode, reply));
            }),
        )?;
        let (errcode, reply) = rx.await.map_err(|err| ErrorCode::Unexpected {
            error_message: format!("Channel error: {}", err),
        })?;
        if errcode != (ErrorCode::Success {}) {
            return Err(errcode);
        }
        Ok(reply)
    }

    // `close()` is used in Kotlin to destroy Uniffi object so we rename it here
    #[uniffi::method(name = "close_pool")]
    pub async fn close(&self) -> Result<(), ErrorCode> {
        _ = self.pool.write().await.take();
        Ok(())
    }
}
