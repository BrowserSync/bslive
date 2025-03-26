#!/bin/bash

# process_tree_test.sh
# Script to create a tree of child processes for testing cleanup behavior

# Function to create a child process that will spawn its own children
create_process_tree() {
    local depth=$1
    local branch_factor=$2
    local sleep_time=$3
    local parent_pid=$$

    echo "[PID $$] Process at depth $depth started, parent: $parent_pid"

    # Base case: if we've reached max depth, just sleep
    if [ $depth -eq 0 ]; then
        echo "[PID $$] Leaf process sleeping for $sleep_time seconds"
        sleep $sleep_time
        echo "[PID $$] Leaf process exiting"
        exit 0
    fi

    # Create child processes
    for i in $(seq 1 $branch_factor); do
        # Fork a child process
        {
            echo "[PID $$] Creating child $i at depth $((depth-1))"
            create_process_tree $((depth-1)) $branch_factor $sleep_time
        } &

        # Store child PID
        child_pid=$!
        echo "[PID $$] Created child with PID $child_pid"
    done

    # Parent process sleeps longer than children
    echo "[PID $$] Parent process at depth $depth sleeping for $((sleep_time * 2)) seconds"
    sleep $((sleep_time * 2))
    echo "[PID $$] Parent process at depth $depth exiting"
}

# Print usage information
print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Creates a tree of child processes for testing process cleanup behavior."
    echo ""
    echo "Options:"
    echo "  -d, --depth INT        Depth of the process tree (default: 3)"
    echo "  -b, --branch INT       Number of children each process creates (default: 2)"
    echo "  -s, --sleep INT        Base sleep time in seconds (default: 10)"
    echo "  -o, --orphan           Create orphaned process tree (parent exits immediately)"
    echo "  -h, --help             Display this help message"
    echo ""
    echo "Example:"
    echo "  $0 -d 2 -b 3 -s 5      Create a tree with depth 2, branching factor 3,"
    echo "                        and sleep time of 5 seconds"
}

# Default values
DEPTH=3
BRANCH_FACTOR=2
SLEEP_TIME=10
ORPHAN=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--depth)
            DEPTH="$2"
            shift 2
            ;;
        -b|--branch)
            BRANCH_FACTOR="$2"
            shift 2
            ;;
        -s|--sleep)
            SLEEP_TIME="$2"
            shift 2
            ;;
        -o|--orphan)
            ORPHAN=true
            shift
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Print the process tree configuration
echo "Process Tree Configuration:"
echo "- Depth: $DEPTH"
echo "- Branch Factor: $BRANCH_FACTOR"
echo "- Sleep Time: $SLEEP_TIME seconds"
echo "- Orphan Mode: $ORPHAN"
echo "- Root PID: $$"
echo ""

# Create process ID file to help with cleanup
echo $$ > /tmp/process_tree_test_$$.pid
echo "PID file created at /tmp/process_tree_test_$$.pid"

# Register cleanup handler for SIGINT
trap cleanup_handler INT
cleanup_handler() {
    echo "Received interrupt signal, cleaning up process tree..."
    pkill -P $$
    rm -f /tmp/process_tree_test_$$.pid
    exit 0
}

# Start the process tree
if [ "$ORPHAN" = true ]; then
    # Create a detached process group that will be orphaned
    echo "Creating orphaned process tree..."
    {
        # Detach from parent's process group
        setsid create_process_tree $DEPTH $BRANCH_FACTOR $SLEEP_TIME
    } &

    echo "Parent process exiting immediately, orphaning the process tree"
    echo "To kill the orphaned process tree, use: kill -TERM -\$(cat /tmp/process_tree_test_$$.pid)"
    exit 0
else
    # Create normal process tree
    echo "Creating normal process tree..."
    create_process_tree $DEPTH $BRANCH_FACTOR $SLEEP_TIME
fi

# Clean up PID file
rm -f /tmp/process_tree_test_$$.pid
echo "Process tree test complete."