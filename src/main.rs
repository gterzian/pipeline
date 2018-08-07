use std::collections::VecDeque;
use std::sync::mpsc::{Sender, channel};
use std::thread;


fn generate(num_chan: Sender<u8>) {
    let mut num = 2;
    let _ = thread::Builder::new().spawn(move || {
        while let Ok(_) = num_chan.send(num) {
            num = num + 1;
        }
    });
}

fn square(result_chan: Sender<u8>) -> Sender<u8> {
    let (chan, port) = channel();
    let _ = thread::Builder::new().spawn(move || {
        for num in port {
            let _ = result_chan.send(num * num);
        }
    });
    chan
}

fn merge(merged_result_chan: Sender<u8>) -> Sender<u8> {
    let (chan, port) = channel();
    let _ = thread::Builder::new().spawn(move || {
        for squared in port {
            let _ = merged_result_chan.send(squared);
        }
    });
    chan
}

#[test]
fn test_run_pipeline() {
    let (results_chan, results_port) = channel();
    let (gen_chan, gen_port) = channel();
    let merge_chan = merge(results_chan);
    {
        let mut square_workers: VecDeque<Sender<u8>> = vec![square(merge_chan.clone()),
                                                            square(merge_chan)]
                                                           .into_iter()
                                                           .collect();
        generate(gen_chan);
        for num in gen_port {
            let worker = square_workers.pop_front().unwrap();
            let _ = worker.send(num);
            square_workers.push_back(worker);
            if num == 3 {
                // Dropping the gen_chan, stopping the generator.
                break;
            }
        }
    }
    for _result in results_port {
        // receiving results...
    }
}
