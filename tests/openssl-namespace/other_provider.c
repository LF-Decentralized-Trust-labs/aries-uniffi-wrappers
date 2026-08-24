int collision_crypto(int value) {
    return value + 1000;
}

int collision_ssl(int value) {
    return value + 2000;
}

int other_provider_value(void) {
    return collision_crypto(1) + collision_ssl(1);
}
