void __syncthreads();

extern "C" __global__ void barrier_kernel(int *x) {
  __syncthreads();
  x[threadIdx.x] = threadIdx.x;
}
