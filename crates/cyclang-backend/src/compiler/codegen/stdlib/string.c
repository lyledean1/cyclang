#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *buffer;
    int32_t length;
    int32_t maxlen;
    int32_t factor;
} StringType;

void stringPrint(StringType *this) {
    printf("\"%s\"\n", this->buffer);
}

void stringCreateDefault(StringType *this) {
    this->buffer = NULL;
    this->length = 0;
    this->maxlen = 0;
    this->factor = 16;  // Default preallocation factor
}

void stringDelete(StringType *this) {
    if (this->buffer != NULL) {
        free(this->buffer);
        this->buffer = NULL; // Ensure the pointer is set to NULL after freeing
    }
}

void stringResize(StringType *this, int new_size) {
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

void stringAddChar(StringType *this, char value) {
    if (this->length == this->maxlen) {
        int new_size = this->maxlen + this->factor;
        stringResize(this, new_size);
    }
    this->buffer[this->length] = value; // Add the character
    this->length++;                     // Increment the length
}

void stringAdd(StringType *this, const StringType *other) {
    for (int i = 0; i < other->length; i++) {
        stringAddChar(this, other->buffer[i]);
    }
}

StringType* stringInit(const char *data) {
    StringType *this = malloc(sizeof(StringType));
    int len = strlen(data);
    stringCreateDefault(this);
    this->buffer = (char *)malloc(len + 1);
    if (this->buffer) {
        memcpy(this->buffer, data, len);
        this->buffer[len] = '\0';
        this->length = len;
        this->maxlen = len;
    }
    return this;
}
