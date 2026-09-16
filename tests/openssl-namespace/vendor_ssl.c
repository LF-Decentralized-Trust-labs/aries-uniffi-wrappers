int collision_crypto(int value);

int collision_ssl(int value) {
    return collision_crypto(value) + 20;
}
