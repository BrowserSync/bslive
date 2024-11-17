set -euxo pipefail

if ! git diff-index --quiet HEAD --; then
    echo "Cannot version, there are changes in the working tree."
    exit 1
fi


if [ -z "$1" ]; then
    echo "First argument (version) is required."
    exit 1
fi

npm version "$1" --no-git-tag-version --workspaces --include-workspace-root
cargo set-version "$1"

echo "did set version to $1"
