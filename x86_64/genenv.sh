#!/bin/sh

echo "CPU_TYPE=\$(uname -m)"> ../profile
fd bin|xargs -I {} echo "export PATH=\$STATIC_GET_HOME/\$CPU_TYPE/{}:\$PATH" >> ../profile
