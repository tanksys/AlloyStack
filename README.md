# AlloyStack

[![CI](https://github.com/anti-entropy123/AlloyStack/actions/workflows/main.yml/badge.svg)](https://github.com/anti-entropy123/AlloyStack/actions/workflows/main.yml)

## Abstract
AlloyStack is a library OS designed for serverless workflow applications. It reduces cold start latency through on-demand loading and optimizes intermediate data transfer overhead via reference passing. We provide user-friendly tools and scripts in the AlloyStack code repository to automate the
build and testing processes. This document guides users in reproducing the experimental results.

## Artifact check-list

* Program: AlloyStack
* Compilation: rustup toolchain (nightly-2023-12-01-x86_64-unknown-linux-gnu) and gcc (11.4.0)
* Run-time environment: Ubuntu 22.04
* Hardware: Intel x86 servers equipped with MPK

## Installation

**Software dependencies**. To build AlloyStack’s LibOS and Rust functions, the toolchain must be installed via rustup. To build C and Python functions, gcc (version 11.4.0) need to be installed. To run automated tests and perform data analysis, [just](https://github.com/casey/just) and python3 need to be installed.

AlloyStack and its benchmarks are [open-sourced](https://github.com/anti-entropy123/AlloyStack) and can be obtained via git clone. The code repository is structured as follows:

```
AlloyStack/
├── libasvisor/ # source code of as-visor
├── as_std/ # source code of as-std
├── common_service/
│   └── ... # as-libos modules
├── user/
│   └── ... # source code of benchmarks
├── isol_config/
│   └── *.json # workflow specification files
├── fs_images/
│   └── *.img # file system images
```


To run a new test application on AlloyStack, user need to develop functions in the `user/` directory. Then, edit the workflow specification files in the `isol_config/` directory to declare how functions compose the workflow, specify dependencies on LibOS modules, and define input parameters for functions. If the workflow involves reading datasets from files, the datasets must also be added to the file system image. Please use the following command to extract the provided image archive, which contains the source code for the Python benchmarks.

```bash
AlloyStack$ just init
```

Additionally, the repository of AlloyStack is integrated with GitHub Actions. Therefore, tools such as [act](https://github.com/nektos/act) can be used locally to quickly run some basic test cases via Docker.

## Evaluation
### Cold start latency

The cold start of AlloyStack can be categorized into two scenarios: enabling and disabling on-demand loading. The approximate cold start latency is measured using the execution time of `hello_world` and `load_all`, respectively. The following script can be used to automate the testing process. 

```bash
AlloyStack$ just cold_start_latency
```

### Intermediate Data Transfer Latency
Users can control the size of the data to be transferred (in bytes) via `user/data_size.config`. The following command can automatically run the test and output key result logs.

```bash
AlloyStack$ just data_transfer_latency
```

### End-to-end latency

The current implementation configures the parallelism of each function through the workflow specification file. Users can generate dataset files tailored to a specific parallelism level using the `scripts/gen_data.py` script. The size of the intermediate data in `Function Chain` can be adjusted via the `function_chain_data_size.config` file located in the `user/` directory. Running the following command will automatically complete data generation, function building, and end-to-end testing for `Word Count`, `Parallel Sorting`, and `Function Chain`.

```bash
AlloyStack$ just end_to_end_latency
```

The breakdown, P99 latency and resource consumption experiments are conducted based on variants of the aforementioned applications. To disable the on-demand loading mechanism, the workflow configuration file needs to be modified (e.g., `map_reduce_load_all.json`). To disable the reference-passing mechanism, the parameter `--features file-based` should be added when building functions. For experiments requiring concurrent request generation, we provide the load generator `p99tester` and `resourcetester`. The following script automatically switches these configuration options and runs the tests.

```bash
AlloyStack$ just breakdown && just p99_latency && just resource_consume
```

## Citation

Please check our paper for technical details and full results.

```
@inproceedings{you2025alloystack,
 author = {You, Jianing and Chen, Kang and Zhao, Laiping and Li, Yiming and Chen, Yichi and Du, Yuxuan and Wang, Yanjie and Wen, Luhang and Hu, Keyang and Li, Keqiu},
 booktitle = {Proceedings of the Twentieth European Conference on Computer Systems},
 doi = {10.1145/3689031.3717490},
 title = {AlloyStack: A Library Operating System for Serverless Workflow Applications},
 year = {2025}
}
```
