extern "C" __global__ void fill(int *x, int n) {
  for (int i = 0; i < n; i = i + 1) {
    x[i] = i;
  }
}
