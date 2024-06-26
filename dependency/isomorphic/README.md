Isomorphic Drivers
===========================

[![Build Status](https://travis-ci.org/rcore-os/isomorphic_drivers.svg?branch=master)](https://travis-ci.org/rcore-os/isomorphic_drivers)

A set of device drivers that should:

1. Work in OS seamlessly
2. Work in user space seamlessly

It accomplishes this by:

1. Call provider methods for memory

Implemented drivers
============================

## Network drivers

- ixgbe: Intel 10Gb Network Adapter 82599
- E1000: Intel E1000

## Block device drivers

- AHCI: Advance Host Controller Interface
