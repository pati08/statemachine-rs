use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use statemachine_derive::statemachine;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::SystemTime;

statemachine!(
    Project {
        doc: String,
    };
    Draft -> Draft: update_doc,
    Draft -> Review: begin_review,
    Review -> ChangeRequested: change_needed,
    ChangeRequested -> Draft: accept,
    ChangeRequested -> Review: reject,
    Review -> Submitted: submit,
    Submitted -> Declined: decline,
    Declined -> Review: restart_review,
    Submitted -> Approved: approve,
);

fn main() {
    let init_time = SystemTime::now();
    let mut doc = String::new();
    println!("Please write the document:");
    std::io::stdin().read_line(&mut doc).unwrap();
    let mut project: Project<Review> = Project {
        _phantom_data: std::marker::PhantomData,
        doc,
    };
    loop {
        match client_review(project.submit(), init_time) {
            ClientResponse::Approved(approved_project) => {
                println!(
                    "The client is happy! The final doc was:\n{}",
                    approved_project.doc
                );
                break;
            }
            ClientResponse::Declined(declined_project) => {
                project = declined_project.restart_review();
                println!("The client is not happy. Please rewrite:");
                project.doc.clear();
                std::io::stdin().read_line(&mut project.doc).unwrap();
            }
        }
    }
}

enum ClientResponse {
    Approved(Project<Approved>),
    Declined(Project<Declined>),
}

fn client_review(project: Project<Submitted>, init_time: SystemTime) -> ClientResponse {
    let seed = hash_seed(&(project.doc.as_str(), init_time));
    let mut rng = StdRng::seed_from_u64(seed);

    let client_approval: f32 = rng.gen();
    if client_approval >= 0.75 {
        ClientResponse::Approved(project.approve())
    } else {
        ClientResponse::Declined(project.decline())
    }
}

fn hash_seed<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}
