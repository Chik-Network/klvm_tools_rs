#!/bin/bash -x

# This script is called from $GIT_ROOT/.github/workflows/build-test.yml
# This script is called while in $GIT_ROOT/chik-blockchain of klvm_tools_rs

. ./venv/bin/activate

python -m pip install --upgrade pip
python -m pip uninstall klvm klvm_rs klvm_tools klvm_tools_rs

git clone https://github.com/Chik-Network/klvm.git --branch=main --single-branch
python -m pip install ./klvm

echo "installing klvm_rs via pip"
pip install klvm_rs

echo "installing klvm_tools for klvm tests"

# Ensure klvm_tools is installed from its own repo.
git clone https://github.com/Chik-Network/klvm_tools.git --branch=main --single-branch
python -m pip install ./klvm_tools

# Install klvm_tools_rs from the directory above.
python -m pip install ..
