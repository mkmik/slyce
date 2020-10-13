#!/bin/bash

set -eux

(
cat <<EOF
![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

EOF

cargo readme
) > README.md
