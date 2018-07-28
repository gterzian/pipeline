use std::collections::VecDeque;
use std::sync::mpsc::{Sender, channel};
use std::thread;


fn generate(numbers: Vec<u8>, num_chan: Sender<u8>) {
    let _ = thread::Builder::new().spawn(move || {
        for num in numbers {
            let _ = num_chan.send(num);
            println!("generated {:?}", num);
        }
        println!("dropping num_chan");
    });
}

fn square(result_chan: Sender<u8>) -> Sender<u8> {
    let (chan, port) = channel();
    let _ = thread::Builder::new().spawn(move || {
        for num in port.iter() {
            println!("square received {:?}", num);
            let _ = result_chan.send(num * num);
        }
        println!("square chan dropped");
    });
    chan
}

fn merge(merged_result_chan: Sender<u8>) -> Sender<u8> {
    let (chan, port) = channel();
    let _ = thread::Builder::new().spawn(move || {
        for squared in port.iter() {
            println!("merge received {:?}", squared);
            let _ = merged_result_chan.send(squared);
        }
        println!("merged chan dropped");
    });
    chan
}

#[test]
fn test_run_pipeline() {
    let (results_chan, results_port) = channel();
    let numbers = vec![2, 3];
    let (gen_chan, gen_port) = channel();
    let merge_chan = merge(results_chan);
    {

        let mut square_workers: VecDeque<Sender<u8>> = vec![square(merge_chan.clone()),
                                                        square(merge_chan.clone())]
                                                        .into_iter()
                                                        .collect();
        generate(numbers, gen_chan);
        for num in gen_port.iter() {
            let worker = square_workers.pop_front().unwrap();
            let _ = worker.send(num);
            square_workers.push_back(worker);
        }
    }
    let mut results = results_port.iter();
    // Two Some, followed by one None.
    assert!(results.next().is_some());
    assert!(results.next().is_some());
    assert!(results.next().is_none());
}
