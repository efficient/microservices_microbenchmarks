Microservices microbenchmarks
=============================
This repository contains the code for the benchmarks presented in our ATC '18 short paper, "Putting the 'Micro' Back in Microservice."

License
-------
The entire contents and history of this repository are distributed under the following license:

	Copyright 2018 Carnegie Mellon University

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

Dependencies
------------
To obtain the results in the paper, we used the following versions:
* Linux 4.13.0 built from upstream
* GCC 7.3.0 from Debian
* Rust 1.23.0 installed using rustup

System components
-----------------
In addition to the experiment driver, this suite consists of the following programs:
* `host`: referred to in the paper as the "dispatcher process"
* `launcher`: referred to in the paper as the "worker process"
* Microservices:
 * `test.rs`: timestamp recorder used for section 2.1 of the paper
 * `sleep.rs`: used for signal predictability study in section 2.2 of the paper
 * `hasher/`: hashing workload used for section 4 of the paper

Modes of operation
------------------
The `host` and `launcher` each have two different modes of operation, selected at compile time.

When doing a `make host`, provide the environment variable `INVOCATION="..."` to specify how microservices should be invoked:
* `launcher` uses worker processes to demonstrate what the paper refers to as "language-based isolation"
* Otherwise, the process launches microservices directly ("process-based isolation") according to the other two possible values:
 * `forkjoin` launches a new microservice process every time a request must be handled ("cold-start invocation" in the paper)
 * `sendmsg` launches a blocking process for each microservice at the start and forwards requests using loopback UDP ("cold-start invocation")

If the dispatcher is configured to use worker processes, do a `make launcher` with `UNLOADING="..."`:
* `cleanup` to `dlopen()`, `dlsym()`, `dlclose()` each time any microservice must be invoked ("cold-start invocation" in the paper)
* `memoize` to keep microservice libraries loaded into the workers after the initial `dlopen()` ("warm-start invocation" in the paper)

Microservices need to be built as shared libraries if using `launcher`, and as executables otherwise.

Data files with reported numbers
--------------------------------
The data files containing the full results from the experimental runs presented in the paper may be downloaded from:
https://github.com/efficient/microservices_microbenchmarks/releases

Each archive contains a script that can be used to easily rerun that experiment.

Invocation latency experiment (section 2.1)
-------------------------------------------
First build the core components:
1. Do a `make distclean` if you already have build artifacts in the checkout.
2. Start by building the `host` and `launcher` (if applicable) as described above.

Now it's time to build the microservice.
To simulate running a large number of diverse microservices, we make 5000 copies of a compiled microservice;
this prevents the kernel from sharing memory pages between them.
To build and copy the necessary microservice, do a

	./mkuls test 5000 bins

for a "process-based isolation" run, or a

	./mkuls libtest.so 5000 libs

for a "language-based isolation" one.

Download and extract the `invocation.tar` archive from the above link.
Notice that there's a folder for each mode of the experiment presented in the paper.
Choose one such folder (we'll call it "src") and the name of a new output folder to be created (we'll call it "dest"), then do:

	src/repeat dest

Once the experiment finishes, the results can be found in a text file within the new folder.

Preemption throughput degradation experiment (section 3)
--------------------------------------------------------
First build the core components using this specific configuration:
1. Do a `make distclean` if you already have build artifacts in the checkout.
2. Run: `make host INVOCATION="launcher"`
3. Run: `make launcher UNLOADING="cleanup"`

Now build the SHA-512 hasher microservice as a shared object: `make hasher/so`

Download and extract the `preemption.tar` archive from the above link.
Decide on the name of some new output folder (here, "out") and do:

	preemption/repeat out

Once the experiment finishes, the results can be found in a series of text files within the new folder.

Signal predictability experiment (section 2.2)
----------------------------------------------
No data files are provided for this experiment.

Start by building the core components using the same steps as in the previous section.
Next, build the microservice using: `make libsleep.so`

Decide on the desired SIGALRM period (which we'll refer to as "quantum"), in microseconds.
Now do:

	./stats 1 taskset 0x1 ./signaling 3

This will run the experiment, then print the absolute values of recorded deviations followed by a statistical summary.
We recommend treating the very first invocation as a warmup round.
