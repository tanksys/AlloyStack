use std::{sync::Arc, thread::sleep, time::Duration};

use clap::{arg, Parser};
use derive_more::Display;

use libasvisor::{
    isolation::{config::IsolationConfig, get_isol, Isolation},
    logger,
};

#[derive(clap::ValueEnum, Clone, Display, Debug)]
pub enum MetricOpt {
    #[display(fmt = "none")]
    None,
    #[display(fmt = "all")]
    All,
    #[display(fmt = "total_dur")]
    TotalDur,
    #[display(fmt = "mem")]
    Mem,
}

impl MetricOpt {
    fn to_analyze(&self) -> libasvisor::MetricOpt {
        match self {
            MetricOpt::None => libasvisor::MetricOpt::None,
            MetricOpt::All => libasvisor::MetricOpt::All,
            MetricOpt::Mem => libasvisor::MetricOpt::Mem,
            MetricOpt::TotalDur => libasvisor::MetricOpt::TotalDur,
        }
    }
}

/// A memory-safe LibOS runtime for serverless.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Config file path.
    #[arg(short, long)]
    files: Vec<String>,

    /// Preload all user modules to speed up.
    #[arg(long, default_value_t = false)]
    preload: bool,

    /// show metrics json.
    #[arg(long, default_value_t = MetricOpt::None)]
    metrics: MetricOpt,

    /// block after workflow execution.
    #[arg(short, long, default_value_t = false)]
    non_exit: bool,
}

fn build_all_isol(args: &Args) -> Vec<Arc<Isolation>> {
    let configs: Vec<_> = if !args.files.is_empty() {
        args.files
            .iter()
            .map(|f| {
                IsolationConfig::from_file(f.into()).unwrap_or_else(|e| {
                    panic!("Read isol_config file failed, file={}, err={}", f, e)
                })
            })
            .collect()
    } else {
        log::debug!("missing arg files");
        vec![
            IsolationConfig::from_file(String::from("base_config.json").into())
                .expect("Open config file failed."),
        ]
    };

    // info!("preload?:{}", args.preload);
    let isols: Vec<_> = configs.iter().map(Isolation::new).collect();

    if args.preload {
        for (idx, isol) in isols.iter().enumerate() {
            isol.preload(configs.get(idx).unwrap())
                .expect("preload failed.")
        }
    }

    isols
}

fn msvisor_start(isol: &Arc<Isolation>) {
    let isol = get_isol(isol.id).expect("isol don't exist?");

    if let Err(e) = isol.run() {
        log::error!("isol{} run failed. err={e:?}", isol.id);
        // if in debug mod, error will lead to exit.
        #[cfg(debug_assertions)]
        panic!("isol{} run failed. err={e:?}", isol.id);
    }
}

#[cfg(feature = "multi_workflow")]
async fn async_msvisor_start(isols: &[Arc<Isolation>]) {
    let mut join_handles: Vec<_> = isols
        .iter()
        .map(|isol| {
            let isol = Arc::clone(isol);
            let isol_start_with_thread = move || msvisor_start(&isol);

            Some(tokio::task::spawn_blocking(move || {
                isol_start_with_thread()
            }))
        })
        .collect();

    for join_handle in join_handles.iter_mut() {
        let join_handle = join_handle.take().unwrap();
        join_handle.await.expect("join failed");
    }
}

fn main() {
    logger::init();
    let args = Args::parse();
    let isols = build_all_isol(&args);

    #[cfg(feature = "multi_workflow")]
    {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_stack_size(8 * 1024 * 1024)
            .build()
            .expect("build tokio runtime failed?");
        runtime.block_on(async_msvisor_start(&isols))
    }
    #[cfg(not(feature = "multi_workflow"))]
    {
        if isols.len() > 1 {
            panic!("enable feature 'multi_workflow' to support multi --files")
        }
        msvisor_start(&isols[0])
    }

    for isol in &isols {
        log::debug!(
            "isol{} has strong count={}",
            isol.id,
            Arc::strong_count(isol)
        );

        if !matches!(args.metrics, MetricOpt::None) {
            isol.metric.analyze(&args.metrics.to_analyze());
        }
    }

    if args.non_exit {
        loop {
            sleep(Duration::from_secs(1));
        }
    }
}
