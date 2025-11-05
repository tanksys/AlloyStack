## Testing a Workflow: map_reduce Example

To test a workflow in AlloyStack user need to go through the following steps. The same methodology applies to any workflow (e.g., Parallel Sort, Function Chain).

### Build System Services
```
# Build all required Libos services
AlloyStack$ just all_libos
```
This command compiles essential system services: `fdtab` `stdio` `mm` `fatfs` `time`

### Build Workflow Functions
```
# Build each function in the workflow
AlloyStack$ just rust_func file_reader
AlloyStack$ just rust_func mapper
AlloyStack$ just rust_func reducer
```
These commands compile the workflow's functional components: `file_reader` `mapper` `reducer` 

### Execute the Workflow
```
# Run the map_reduce workflow
AlloyStack$ target/release/asvisor --files isol_config/map_reduce.json
```
### Workflow Execution Process

#### Configuration (map_reduce.json)
```
{
  "groups": [
    // Stage 1: 3 file readers with different inputs
    {
      "list": [
        {"name": "file_reader", "args": {"slot_name": "part-0", "input_file": "fake_data_0.txt"}},
        {"name": "file_reader", "args": {"slot_name": "part-1", "input_file": "fake_data_1.txt"}},
        {"name": "file_reader", "args": {"slot_name": "part-2", "input_file": "fake_data_2.txt"}}
      ]
    },
    // Stage 2: 3 parallel mappers
    {
      "list": ["mapper", "mapper", "mapper"],
      "args": {"reducer_num": "4"}
    },
    // Stage 3: 4 parallel reducers
    {
      "list": ["reducer", "reducer", "reducer", "reducer"],
      "args": {"mapper_num": "3"}
    }
  ]
}

# Stage 1: File Reading (Parallel)
file_reader: slot_name: part-0
file_reader: read_size=9881
file_reader: slot_name: part-1
file_reader: read_size=9951
file_reader: slot_name: part-2
file_reader: read_size=9903
...
# Stage 2: Map Processing (Parallel)
mapper: The sum of all values is: 1194
mapper: the counter nums is 825
mapper: shuffle end, cost 0ms
...
# Stage 3: Reduce Processing (Parallel)
reducer0 has counted 372 words
reducer1 has counted 361 words
reducer2 has counted 350 words
reducer3 has counted 362 words
...
```

### Testing Other Workflows
The same 3-step process applies to any workflow:

1.Build services: `just all_libos`

2.Build functions: `just rust_func <function_name>`

3.Execute workflow: `target/release/asvisor --files isol_config/<workflow_name>.json`