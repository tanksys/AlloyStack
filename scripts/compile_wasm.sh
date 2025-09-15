#!/usr/bin/env bash
set -euo pipefail

# Activate environment：export CPP="/opt/wasi-sdk/bin/clang++" &&  export CC="/opt/wasi-sdk/bin/clang"
# Before compiling other modules: unset CC && export CC=/usr/bin/clang && echo "CC is now: $CC" 

# Help information
show_help() {
    echo "Usage: $0 [OPTIONS] [COMPONENT...]"
    echo ""
    echo "Compile WebAssembly modules, supporting two workflows: wordcount and parallel_sort."
    echo "If no components are specified, all components will be compiled."
    echo ""
    echo "Options:"
    echo "  -h, --help            Show help information"
    echo "  -c, --concurrency N   Specify concurrency level, available values: 1, 3, 5 (default: 1)"
    echo "  -m, --mode MODE       Specify build mode, available values: debug, release (default: release)"
    echo "  -w, --workflow NAME   Specify workflow, available values: wordcount, parallel_sort, all (default: all)"
    echo "  --cflags FLAGS        Custom CFLAGS compilation parameters, overriding default parameters"
    echo ""
    echo "Components:"
    echo "  wordcount workflow components: mapper, reducer"
    echo "  parallel_sort workflow components: spliter, sorter, merger, checker"
    echo ""
    echo "Examples:"
    echo "  # Compile all components with default concurrency 1 and release mode"
    echo "  $0"
    echo ""
    echo "  # Compile only wordcount workflow components"
    echo "  $0 --workflow wordcount"
    echo ""
    echo "  # Compile only parallel_sort workflow components with concurrency 3"
    echo "  $0 --workflow parallel_sort --concurrency 3"
    echo ""
    echo "  # Compile specific components"
    echo "  $0 mapper reducer"
    echo ""
    echo "  # Compile all components in debug mode"
    echo "  $0 --mode debug"
}

# Default parameters
CONCURRENCY=1
BUILD_MODE="release"
WORKFLOW="all"
TARGET_DIR="x86_64-unknown-none"
CUSTOM_CFLAGS=""
# Initialize components array
COMPONENTS=()

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -c|--concurrency)
            CONCURRENCY=$2
            shift 2
            ;;
        -m|--mode)
            BUILD_MODE=$2
            shift 2
            ;;
        -w|--workflow)
            WORKFLOW=$2
            shift 2
            ;;
        --cflags)
            CUSTOM_CFLAGS=$2
            shift 2
            ;;
        *)
            # Collect component arguments
            COMPONENTS+=($1)
            shift
            ;;
    esac

done

# Validate parameters
if [[ ! $CONCURRENCY =~ ^[135]$ ]]; then
    echo "Error: Concurrency must be 1, 3 or 5" >&2
    exit 1
fi

if [[ ! $BUILD_MODE =~ ^(debug|release)$ ]]; then
    echo "Error: Build mode must be debug or release" >&2
    exit 1
fi

if [[ ! $WORKFLOW =~ ^(wordcount|parallel_sort|all)$ ]]; then
    echo "Error: Workflow must be wordcount, parallel_sort or all" >&2
    exit 1
fi

# Set workflow components
if [[ ${#COMPONENTS[@]} -eq 0 ]]; then
    case $WORKFLOW in
        wordcount)
            COMPONENTS=(mapper reducer)
            ;;
        parallel_sort)
            COMPONENTS=(spliter sorter merger checker)
            ;;
        all)
            COMPONENTS=(mapper reducer spliter sorter merger checker)
            ;;
    esac
fi

# Check component validity
VALID_COMPONENTS=(mapper reducer spliter sorter merger checker)
for comp in "${COMPONENTS[@]}"; do
    if [[ ! " ${VALID_COMPONENTS[*]} " =~ " $comp " ]]; then
        echo "Error: Invalid component name: $comp" >&2
        echo "Valid component names: ${VALID_COMPONENTS[*]}" >&2
        exit 1
    fi
    
    # Check workflow compatibility
    if [[ $WORKFLOW == "wordcount" && ! " mapper reducer " =~ " $comp " ]]; then
        echo "Error: Component $comp does not belong to the wordcount workflow" >&2
        exit 1
    elif [[ $WORKFLOW == "parallel_sort" && ! " spliter sorter merger checker " =~ " $comp " ]]; then
        echo "Error: Component $comp does not belong to the parallel_sort workflow" >&2
        exit 1
    fi
done

# Set compilation environment variables
export CPP="/opt/wasi-sdk/bin/clang++"
export CC="/opt/wasi-sdk/bin/clang"

echo "Using compilation environment: CC=$CC, CPP=$CPP"
echo "Build mode: $BUILD_MODE"
echo "Concurrency: C$CONCURRENCY"
echo "Components to compile: ${COMPONENTS[*]}"

# Set cargo parameters
CARGO_FLAGS="--target $TARGET_DIR"
if [[ $BUILD_MODE == "release" ]]; then
    CARGO_FLAGS+=" --release"
fi

# Compilation function
compile_component() {
    local component=$1
    local source_file="$component.c"
    local wasm_file="$component.wasm"
    local cwasm_file="$component.cwasm"
    local cargo_dir="wasmtime_$component"
    local output_dir="target/$TARGET_DIR/$BUILD_MODE"
    local lib_name="libwasmtime_$component.so"
    local symlink_path="./target/$BUILD_MODE/$lib_name"
    
    echo "\n===== Compiling $component ====="
    
    # Enter component directory
    cd "$WORKSPACE/user/$cargo_dir"
    echo "Current directory: $(pwd)"
    
    # Set compilation parameters
    if [[ -n "$CUSTOM_CFLAGS" ]]; then
        # If custom CFLAGS is provided, use custom parameters
        CFLAGS="$CUSTOM_CFLAGS"
        echo "Using custom compilation parameters: $CFLAGS"
    else
        # Otherwise use default parameters
        case $component in
            mapper|reducer)
                # Wordcount workflow component parameters
                if [[ $CONCURRENCY -eq 1 ]]; then
                    CFLAGS="-DMAX_WORD_LENGTH=20 -DMAX_WORDS=1000 -DMAX_SLOT_NUM=100 -DMAX_BUFFER_SIZE=50000"
                elif [[ $CONCURRENCY -eq 3 ]]; then
                    CFLAGS="-DMAX_WORD_LENGTH=20 -DMAX_WORDS=5000 -DMAX_SLOT_NUM=100 -DMAX_BUFFER_SIZE=250000"
                elif [[ $CONCURRENCY -eq 5 ]]; then
                    CFLAGS="-DMAX_WORD_LENGTH=20 -DMAX_WORDS=10000 -DMAX_SLOT_NUM=100 -DMAX_BUFFER_SIZE=500000"
                fi
                ;;
            spliter|sorter|merger|checker)
                 # Parallel sort workflow component parameters
                if [[ $CONCURRENCY -eq 1 ]]; then
                    CFLAGS="-DMAX_ARRAY_LENGTH=1600000 -DMAX_BUFFER_SIZE=15000000"
                elif [[ $CONCURRENCY -eq 3 ]]; then
                    CFLAGS="-DMAX_ARRAY_LENGTH=8000000 -DMAX_BUFFER_SIZE=80000000"
                elif [[ $CONCURRENCY -eq 5 ]]; then
                    CFLAGS="-DMAX_ARRAY_LENGTH=8000000 -DMAX_BUFFER_SIZE=80000000"
                fi
                ;;
        esac
        echo "Compilation parameters: $CFLAGS"
    fi
    
    # Compile C to WASM
    echo "Compiling C to WASM..."
    $CC $source_file -o $wasm_file -O3 $CFLAGS
    
    # Compile WASM to CWASM
    echo "Compiling WASM to CWASM..."
    wasmtime compile --target $TARGET_DIR -W threads=n,tail-call=n $wasm_file
    
    # Compile Rust code and create shared library
    echo "Compiling Rust code and creating shared library..."
    cargo build $CARGO_FLAGS
    
    cc -Wl,--gc-sections -nostdlib \
        -Wl,--whole-archive \
        $output_dir/lib$cargo_dir.a \
        -Wl,--no-whole-archive \
        -shared \
        -o $output_dir/$lib_name
    
    # Create or update symbolic link
    echo "Updating symbolic link..."
    if [ -L "$symlink_path" ]; then
        rm "$symlink_path"
    fi
    ln -s "$(pwd)/$output_dir/$lib_name" "$symlink_path"
    
    echo "$component compilation completed!"
    
    # Return to script directory
    cd "$WORKSPACE"
}

# Set workspace directory
WORKSPACE="$(cd "$(dirname "$0")" && cd .. && pwd)"
cd "$WORKSPACE"

# Compile all specified components
for component in "${COMPONENTS[@]}"; do
    compile_component $component
    if [ $? -ne 0 ]; then
        echo "$component compilation failed!" >&2
        exit 1
    fi
done

unset CPP && export CPP=/usr/bin/clang++ && unset CC && export CC=/usr/bin/clang

echo "All components compiled successfully!"