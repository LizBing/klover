use klover::{class_loader::bootstrap_cld::BootstrapCLD, runtime::{arguments::Arguments, vm::vm_init}};

const TEST_CLASS_PATH: &str = "../test_data/classes";

#[test]
fn test_loading() {
    let args = Arguments {
        bs_class_path: TEST_CLASS_PATH.into(),
        xmx: 64 * 1024 * 1024
    };

    vm_init(args);

    let klass = BootstrapCLD::find_class("SimpleAddition");
    println!("{:?}", klass);
}
