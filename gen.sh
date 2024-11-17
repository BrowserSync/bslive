set -e  # Exit immediately if a command exits with a non-zero status.
set -u  # Treat unset variables as an error when substituting.
set -x  # Print commands and their arguments as they are executed.
set -o pipefail  # The return value of a pipeline is the status of the last command to exit with a non-zero status.

typeshare crates/bsnext_dto --lang=typescript --output-file=generated/dto.ts
npm run schema --workspace=generated
npm run build:client