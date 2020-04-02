use cloudflare::endpoints::account::{
    Account,
    list_accounts::ListAccountsParams,
    ListAccounts,
};
use cloudflare::framework::{
    apiclient::ApiClient,
    HttpApiClient,
    OrderDirection,
};
use tabular::Row;

use crate::commands::table_from_cols;

pub fn list(api: &HttpApiClient, page: u32, limit: u32) {
    let response = api.request(&ListAccounts {
        params: Some(ListAccountsParams {
            page: Some(page),
            per_page: Some(limit),
            direction: Some(OrderDirection::Ascending),
        })
    });

    match response {
        Ok(success) => {
            let list: Vec<Account> = success.result;
            let columns = vec![
                "ID",
                "NAME"
            ];
            let mut table = table_from_cols(columns);

            for acc in list {
                table.add_row(Row::new()
                    .with_cell(acc.id)
                    .with_cell(acc.name));
            }
            print!("{}", table);
        }
        Err(e) => println!("{:?}", e)
    }
}
