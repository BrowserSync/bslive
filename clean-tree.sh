set -euxo pipefail

if ! git diff-index --quiet HEAD --; then
    echo "Cannot continue, there are changes in the working tree."
    git diff --name-status
    exit 1
fi