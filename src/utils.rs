use crate::*;

pub async fn call_view(
    contract: &str,
    method_name: String,
    args: FunctionArgs,
) -> Result<RpcQueryResponse, Box<dyn std::error::Error>> {
    let client = JsonRpcClient::connect("https://rpc.testnet.near.org");

    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: contract.parse()?,
            method_name: method_name,
            args: args,
        },
    };

    let response = client.call(request).await?;
    Ok(response)
}
