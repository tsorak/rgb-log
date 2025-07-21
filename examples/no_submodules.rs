use rgb_log::Log;

fn main() {
    let log = Log::builder().build();

    // Without any registered submodule names,
    // shorter names will not align with longer ones
    // to the right

    log.info("I am the default Log built with Log::builder().build()");

    log.submodule("main").info("blup");
    log.submodule("long_module_name").info("blip");
    log.submodule("xd").info("bloop");

    log.debug("Now with submodules added:");

    let log = Log::builder()
        .with_submodule_names(["main", "long_module_name", "xd"])
        .build();

    log.submodule("main").info("blup");
    log.submodule("long_module_name").info("blip");
    log.submodule("xd").info("bloop");
}
