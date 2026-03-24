extern "C" __global__ void saxpy(int *x, int *y, int n) {
  __shared__ int tile[32];
  int i = blockIdx.x * blockDim.x + threadIdx.x;
  if (i < n) {
    tile[threadIdx.x] = x[i];
    y[i] = tile[threadIdx.x] + y[i];
  }
}
