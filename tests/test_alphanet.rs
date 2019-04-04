use std::thread;
use std::time::Duration;

use primitives::transaction::{SignedTransaction, TransactionBody};
use testlib::alphanet_utils::create_nodes;
use testlib::alphanet_utils::Node;
use testlib::alphanet_utils::sample_two_nodes;
use testlib::alphanet_utils::wait;
use network::proxy::ProxyHandler;
use std::sync::Arc;
use network::proxy::benchmark::BenchmarkHandler;

fn run_multiple_nodes(num_nodes: usize, num_trials: usize, test_prefix: &str, test_port: u16) {
    // Add proxy handlers to the pipeline.
    let proxy_handlers: Vec<Arc<ProxyHandler>> = vec![
        Arc::new(BenchmarkHandler::new())
    ];

    let (init_balance, account_names, mut nodes) = create_nodes(num_nodes, test_prefix, test_port, proxy_handlers);

    let mut nodes: Vec<Box<Node>> = nodes.drain(..).map(|cfg| Node::new(cfg)).collect();
    for i in 0..num_nodes {
        nodes[i].start();
    }

    // Execute N trials. In each trial we submit a transaction to a random node i, that sends
    // 1 token to a random node j. We send transaction to node Then we wait for the balance change to propagate by checking
    // the balance of j on node k.
    let mut expected_balances = vec![init_balance; num_nodes];
    let trial_duration = 60000;
    for trial in 0..num_trials {
        println!("TRIAL #{}", trial);
        let (i, j) = sample_two_nodes(num_nodes);
        let (k, r) = sample_two_nodes(num_nodes);
        let nonce = nodes[i].get_account_nonce(&account_names[i]).unwrap_or_default() + 1;
        let tx_body = TransactionBody::send_money(
            nonce,
            account_names[i].as_str(),
            account_names[j].as_str(),
            1,
        );
        let transaction = SignedTransaction::new(nodes[i].signer().sign(&tx_body.get_hash()), tx_body);
        nodes[k].add_transaction(transaction).unwrap();
        expected_balances[i] -= 1;
        expected_balances[j] += 1;

        wait(
            || expected_balances[j] == nodes[r].view_balance(&account_names[j]).unwrap(),
            1000,
            trial_duration,
        );
        thread::sleep(Duration::from_millis(500));
    }
}

#[test]
fn test_4_10_multiple_nodes() {
    run_multiple_nodes(4, 10, "4_10", 3200);
}

// TODO(#718)
//#[test]
//fn test_7_10_multiple_nodes() {
//    run_multiple_nodes(7, 10, "7_10", 3300);
//}
