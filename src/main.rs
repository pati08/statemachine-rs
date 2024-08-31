use rand::random;
use statemachine_derive::statemachine;

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
    let mut doc = String::new();
    println!("Please write the document:");
    std::io::stdin().read_line(&mut doc).unwrap();
    let mut project: Project<Review> = Project {
        _phantom_data: std::marker::PhantomData,
        doc,
    };
    loop {
        match client_review(project.submit()) {
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

fn client_review(project: Project<Submitted>) -> ClientResponse {
    let client_approval: f32 = random();
    if client_approval >= 0.5 {
        ClientResponse::Approved(project.approve())
    } else {
        ClientResponse::Declined(project.decline())
    }
}
