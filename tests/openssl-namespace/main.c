int anoncreds_value(void);
int other_provider_value(void);

int main(void) {
    if (anoncreds_value() != 31) {
        return 1;
    }
    if (other_provider_value() != 3002) {
        return 2;
    }
    return 0;
}
