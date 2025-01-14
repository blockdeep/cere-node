#!/bin/sh

# Prevent committing badly formatted code

cargo +nightly-2024-03-12 fmt -- --check
if [ $? -ne 0 ]; then
	echo "Run \`cargo fmt\` to fix formatting issues before committing."
	exit 1
fi

dprint check
if [ $? -ne 0 ]; then
	echo "Run \`dprint fmt\` to fix formatting issues before committing."
	exit 1
fi
