# hvm-bench

Benchmarks [HVM2][1] versions.

Based on [hvm-compare-perf][2].

## Usage
```
Usage: hvm-bench bench [OPTIONS]

Options:
      --repo-dir <REPO_DIR>  Path to local hvm repo to benchmark [default: ./hvm]
  -r, --revs <REVS>          Which revisions in the remote repository to benchmark
      --timeout <TIMEOUT>    Timeout in seconds [default: 60]
  -h, --help                 Print help
```
For example,

```sh
hvm-bench bench \
  --repo-dir local_hvm_dir/ \
  --revs main \
  --revs another_remote_rev \
  --timeout 20
```

## Output
```
interpreted
===========

file              runtime     main          a43dcfa57c9d
========================================================
sum_rec           rust            14.018 s      15.381 s
                  c                7.324 s       7.109 s
                  cuda             7.324 s       7.109 s
--------------------------------------------------------
sum_rec           rust            14.018 s      15.381 s
                  c                7.324 s       7.109 s
                  cuda             7.324 s       7.109 s
--------------------------------------------------------
sum_rec           rust            14.018 s      15.381 s
                  c                7.324 s       7.109 s
                  cuda             7.324 s       7.109 s
--------------------------------------------------------
sum_rec           rust            14.018 s      15.381 s
                  c                7.324 s       7.109 s
                  cuda             7.324 s       7.109 s
--------------------------------------------------------

compiled
===========

file              runtime     main          a43dcfa57c9d
========================================================
sum_rec           cuda            14.018 s      15.381 s
                  c                7.324 s       7.109 s
--------------------------------------------------------
sum_rec           cuda            14.018 s      15.381 s
                  c                7.324 s       7.109 s
--------------------------------------------------------
sum_rec           cuda            14.018 s      15.381 s
                  c                7.324 s       7.109 s
--------------------------------------------------------
sum_rec           cuda            14.018 s      15.381 s
                  c                7.324 s       7.109 s
--------------------------------------------------------
```


[1]: https://github.com/HigherOrderCO/hvm
[2]: https://github.com/HigherOrderCO/hvm-compare-perf
