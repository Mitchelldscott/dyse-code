#ifndef CURO_LINALG
#define CURO_LINALG

/*
	Vector ops
*/

float nd_norm(float* v, int n);
float cross_product2D(float* a, float* b);
void rotate2D(float* v, float* v_tf, float angle);
float vector_product(float* a, float* b, int n);
void weighted_vector_addition(float* a, float* b, float k1, float k2, int n, float* output);

/*
	Angular ops
*/

float wrap_angle(float angle);


#endif