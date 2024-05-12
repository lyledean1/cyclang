#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// * STRING IMPLEMENTATION * // 
typedef struct {
    char *buffer;
    int32_t length;
    int32_t maxlen;
    int32_t factor;
} StringType;

void stringPrint(StringType *this) {
    printf("\"%s\"\n", this->buffer);
}

void stringPrintList(StringType *this) {
    printf("\"%s\"", this->buffer);
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
    this->buffer[this->length] = value;
    this->length++;
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

// * LIST IMPLEMENTATION * //
int32_t* createInt32List(int size) {
    // set sentinel value of -1 hence size + 1
    int32_t* arr = (int32_t*)malloc((size + 1) * sizeof(int32_t));
    arr[size] = -1;
    return arr;
}

int32_t getInt32Value(int32_t* arr, int index) {
    return arr[index];
}

void setInt32Value(int32_t* arr, int32_t value, int index) {
    arr[index] = value;
}

void printInt32List(int32_t* arr) {
    int i = 0;
    printf("[");
    while (arr[i] != -1) {
        if (i != 0) {
            printf(",");
        }
        printf("%d", arr[i]);
        i++;
    }
    printf("]");
}

int64_t* createInt64List(int size) {
    // set sentinel value of -1 hence size + 1
    int64_t* arr = (int64_t*)malloc((size + 1) * sizeof(int64_t));
    arr[size] = -1;
    return arr;
}

int64_t getInt64Value(int64_t* arr, int index) {
    return arr[index];
}

void setInt64Value(int64_t* arr, int64_t value, int index) {
    arr[index] = value;
}

void printInt64List(int64_t* arr) {
    int i = 0;
    printf("[");
    while (arr[i] != -1) {
        if (i != 0) {
            printf(",");
        }
        printf("%lld", arr[i]);
        i++;
    }
    printf("]");
}

