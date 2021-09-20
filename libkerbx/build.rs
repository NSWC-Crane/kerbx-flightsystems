//use glob;
use krpc_mars_terraformer;
use protoc_rust;
use protoc_rust::Customize;

fn main() {
    // Tell cargo to re-run this script only when json files in services/
    // have changed. You can choose to omit this step if you want to
    // re-generate services every time.
    for path in glob::glob("services/*.json")
        .unwrap()
        .filter_map(Result::ok)
    {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    krpc_mars_terraformer::run("services/", "src/").expect("Could not terraform Mars :(");

    // Generate our protobuf files
    println!("cargo:rerun-if-changed=protos/kerbx.proto");
    println!("cargo:rerun-if-changed=src/kerbx.rs");

    protoc_rust::Codegen::new()
        .out_dir("src/")
        .inputs(&["protos/kerbx.proto"])
        .include("protos")
        .customize(Customize {
            serde_derive: Some(true),
            ..Default::default()
        })
        .run()
        .expect("Running protoc failed.");
}
