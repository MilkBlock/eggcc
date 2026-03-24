#include "helper.cuh"

extern "C" __global__ void include_kernel(int *x) {
  x[threadIdx.x] = load_bias();
}
