#!/bin/bash
set -e

mkdir -p save{1,2,3}/{clients,planes,terrain_chunks}

../build/native/move-to-spawn save0/clients save1/clients
../build/native/2016-08-17-new-tools save1/clients save2/clients
../build/native/init-world-dat save2/world.dat
../build/native/init-plane save2/planes/1.plane Limbo 1
../build/native/init-plane save2/planes/2.plane 'Everfree Forest' 2
../build/native/renumber_stable_ids save2 save3
../build/native/check-data-names save3 ../dist/server

echo
echo Done

