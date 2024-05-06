#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *buffer;
    int32_t length;
    int32_t maxlen;
    int32_t factor;
} String;

void stringCreateDefault(String *this) {
    this->buffer = NULL;
    this->length = 0;
    this->maxlen = 0;
    this->factor = 16;  // Default preallocation factor
}

void stringDelete(String *this) {
    if (this->buffer != NULL) {
        free(this->buffer);
        this->buffer = NULL; // Ensure the pointer is set to NULL after freeing
    }
}

void stringResize(String *this, int new_size) {
    char *new_buffer = (char *)malloc(new_size);
    if (new_buffer) {
        if (this->buffer) {
            memcpy(new_buffer, this->buffer, this->length);
            free(this->buffer);
        }
        this->buffer = new_buffer;
        this->maxlen = new_size;
    } else {
        fprintf(stderr, "Failed to allocate memory\n");
    }
}

void stringAddChar(String *this, char value) {
    if (this->length == this->maxlen) {
        int new_size = this->maxlen + this->factor;
        stringResize(this, new_size);
    }
    this->buffer[this->length] = value; // Add the character
    this->length++;                     // Increment the length
}

void stringPrint(String *this) {
    printf("String: %s\n", this->buffer);
}

void boolToStrC() {
    String myString;
    stringCreateDefault(&myString);
    stringAddChar(&myString, 'A');
    stringAddChar(&myString, 'B');
    stringAddChar(&myString, 'C');
    stringPrint(&myString);
    stringDelete(&myString);
}

