use ::kserd::Kserd;
use libloading::{Library, Symbol};
use std::io::{self, Write};
use std::path::Path;

/// We don't type anything here. You must be **VERY** careful to pass through the correct borrow to match the
/// function signature!
type DataFunc<D> = unsafe fn(D) -> Kserd<'static>;

type ExecResult = Result<(Kserd<'static>, Library), &'static str>;

pub(crate) fn exec<P: AsRef<Path>, D, W: Write + Send>(
    library_file: P,
    function_name: &str,
    app_data: D,
    wtr: Option<&mut W>,
) -> ExecResult {
    if let Some(wtr) = wtr {
        exec_and_redirect(library_file, function_name, app_data, wtr)
    } else {
        exec_no_redirect(library_file, function_name, app_data)
    }
}

fn exec_no_redirect<P: AsRef<Path>, Data>(
    library_file: P,
    function_name: &str,
    app_data: Data,
) -> ExecResult {
    let lib = get_lib(library_file)?;
    let func = get_func(&lib, function_name)?;

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { func(app_data) }));

    match res {
        Ok(kserd) => Ok((kserd, lib)),
        Err(_) => Err("a panic occured with evaluation"),
    }
}

fn exec_and_redirect<P: AsRef<Path>, Data, W: Write + Send>(
    library_file: P,
    function_name: &str,
    app_data: Data,
    output_wtr: &mut W,
) -> ExecResult {
    let lib = get_lib(library_file)?;
    let func = get_func(&lib, function_name)?;

    let (tx, rx) = crossbeam_channel::bounded(0);

    let (stdout_gag, stderr_gag) =
        get_gags().map_err(|_| "failed to apply redirect gags on stdout and stderr")?;

    let res = crossbeam_utils::thread::scope(|scope| {
        let jh = if cfg!(debug_assertions) {
            // don't redirect on debug builds, such that dbg!() can print through to terminal for debugging.
            drop(stderr_gag);
            scope.spawn(|_| redirect_output_loop(output_wtr, rx, stdout_gag, std::io::empty()))
        } else {
            scope.spawn(|_| redirect_output_loop(output_wtr, rx, stdout_gag, stderr_gag))
        };

        let res =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { func(app_data) }));

        tx.send(()).expect("sending signal to stop gagging failed");
        jh.join()
            .expect("joining gagging thread failed")
            .expect("failed to redirect output loop");
        res
    });

    let res = res.map_err(|_| "crossbeam scoping failed")?;

    match res {
        Ok(kserd) => Ok((kserd, lib)),
        Err(_) => Err("a panic occured with evaluation"),
    }
}

fn get_lib<P: AsRef<Path>>(path: P) -> Result<Library, &'static str> {
    // If segfaults are occurring maybe use this, SIGSEV?
    // This is shown in https://github.com/nagisa/rust_libloading/issues/41
    // let lib: Library =
    // 	libloading::os::unix::Library::open(Some(library_file.as_ref()), 0x2 | 0x1000)
    // 		.unwrap()
    // 		.into();
    Library::new(path.as_ref()).map_err(|e| {
        error!("failed to load library file: {}", e);
        "failed to load library file"
    })
}

fn get_func<'l, Data>(
    lib: &'l Library,
    name: &str,
) -> Result<Symbol<'l, DataFunc<Data>>, &'static str> {
    unsafe {
        lib.get(name.as_bytes())
            .map_err(|_| "failed to find function in library")
    }
}

fn get_gags() -> io::Result<(shh::ShhStdout, shh::ShhStderr)> {
    Ok((shh::stdout()?, shh::stderr()?))
}

fn redirect_output_loop<W: Write, R1: io::Read, R2: io::Read>(
    wtr: &mut W,
    rx: crossbeam_channel::Receiver<()>,
    mut stdout_gag: R1,
    mut stderr_gag: R2,
) -> io::Result<()> {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(2)); // add in some delay as reading occurs to avoid smashing cpu.

        let mut buf = Vec::new();

        // read/write stderr first
        stderr_gag.read_to_end(&mut buf)?;

        stdout_gag.read_to_end(&mut buf)?;

        wtr.write_all(&buf)?;

        match rx.try_recv() {
            Ok(_) => break,                                              // stop signal sent
            Err(crossbeam_channel::TryRecvError::Disconnected) => break, // tx dropped
            _ => (),
        };
    }

    Ok(())
}
