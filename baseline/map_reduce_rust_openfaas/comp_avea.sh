#!/bin/bash

read -p "Enter the number of runs: " num_runs
read -p "Enter the number of instances: " num_instances

tot_cost_time=0

for ((i=1; i<=num_runs; i++))
do
	output=$(python3 client.py $num_instances)

    cost_time=$(echo "$output" | grep "total cost time" | grep -o '[0-9]*\.[0-9]*')

    tot_cost_time=$(echo "$tot_cost_time + $cost_time" | bc)
done

average_cost_time=$(echo "scale=6; $tot_cost_time / $num_runs" | bc)

echo "Average cost time: $average_cost_time ms"
