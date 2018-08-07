use std::collections::VecDeque;
use std::sync::mpsc::{Sender, channel};
use std::thread;


enum PipelineMsg {
    Generated(u8),
    Squared(u8),
    Merged(u8),
}

fn generate(num_chan: Sender<PipelineMsg>) {
    let mut num = 2;
    let _ = thread::Builder::new().spawn(move || {
        while let Ok(_) = num_chan.send(PipelineMsg::Generated(num)) {
            num = num + 1;
        }
    });
}

fn square(merge_chan: Sender<PipelineMsg>) -> Sender<PipelineMsg> {
    let (chan, port) = channel();
    let _ = thread::Builder::new().spawn(move || {
        for msg in port {
            let num = match msg {
                PipelineMsg::Generated(num) => num,
                _ => panic!("unexpected message receiving at square stage"),
            };
            let _ = merge_chan.send(PipelineMsg::Squared(num * num));
        }
    });
    chan
}

fn merge(merged_result_chan: Sender<PipelineMsg>) -> Sender<PipelineMsg> {
    let (chan, port) = channel();
    let _ = thread::Builder::new().spawn(move || {
        for msg in port {
            let squared = match msg {
                PipelineMsg::Squared(num) => num,
                _ => panic!("unexpected message receiving at merge stage"),
            };
            let _ = merged_result_chan.send(PipelineMsg::Merged(squared));
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
        let mut square_workers: VecDeque<Sender<PipelineMsg>> = vec![square(merge_chan.clone()),
                                                                     square(merge_chan)]
                                                                     .into_iter()
                                                                     .collect();
        generate(gen_chan);
        for msg in gen_port {
            let generated_num = match msg {
                PipelineMsg::Generated(num) => num,
                _ => panic!("unexpected message receiving from gen stage"),
            };
            let worker = square_workers.pop_front().unwrap();
            let _ = worker.send(msg);
            square_workers.push_back(worker);
            if generated_num == 3 {
                // Dropping the gen_chan, stopping the generator.
                break;
            }
        }
    }
    for result in results_port {
        match result {
            PipelineMsg::Merged(_) => continue,
            _ => panic!("unexpected result"),
        }
    }
}
