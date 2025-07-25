#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>


// * MACROS * // 
#define DEFINE_GET_VALUE_FUNC(type) \
type get_##type##Value(type* arr, int index) { \
    return arr[index]; \
}

#define DEFINE_SET_VALUE_FUNC(type) \
void set_##type##Value(type* arr, type value, int index) { \
    arr[index] = value; \
}

#define DEFINE_CREATE_VALUE_FUNC(type) \
type* create_##type##List(int size) { \
    type* arr = (type*)malloc((size + 1) * sizeof(type)); \
    arr[size] = -1; \
    return arr; \
} 

DEFINE_CREATE_VALUE_FUNC(int32_t)
DEFINE_GET_VALUE_FUNC(int32_t)
DEFINE_SET_VALUE_FUNC(int32_t)
DEFINE_CREATE_VALUE_FUNC(int64_t)
DEFINE_GET_VALUE_FUNC(int64_t)
DEFINE_SET_VALUE_FUNC(int64_t)

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

bool isStringEqual(StringType *stringOne, StringType* stringTwo) {
    if (stringOne->length != stringTwo->length) {
        return false;
    }
    int i = 0;
    while (i < stringOne->length) {
        if (stringOne->buffer[i] != stringTwo->buffer[i]) {
            return false;
        }
        i++;
    }
    return true;
}

// * LIST IMPLEMENTATION * //
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

int32_t lenInt32List(int32_t* arr) {
    int i = 0;
    while (arr[i] != -1) {
        i++;
    }
    return i;
}

int32_t* concatInt32List(int32_t* arrOne, int32_t* arrTwo) {
    int sizeOne = lenInt32List(arrOne);
    int sizeTwo = lenInt32List(arrTwo);
    // add a -1 terminator
    int *result = (int32_t*)malloc((sizeOne + sizeTwo + 1) * sizeof(int32_t));
    result[sizeOne + sizeTwo] = -1;
    if (result == NULL) {
        printf("Memory allocation failed\n");
        exit(1);
    }

    // Copy over first elements
    for (int i = 0; i < sizeOne; i++) {
        result[i] = arrOne[i];
    }

    // Copy over second elements
    for (int i = 0; i < sizeTwo; i++) {
        result[sizeOne + i] = arrTwo[i];
    }
    return result;
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

StringType** createStringList(int size) {
    // set sentinel value of NULL hence size + 1
    StringType **stringArray = malloc((size + 1) * sizeof(StringType *));
    stringArray[size] = NULL;
    return stringArray;
}

StringType* getStringValue(StringType** arr, int index) {
    return arr[index];
}

void setStringValue(StringType** arr, StringType* value, int index) {
    arr[index] = value;
}

void printStringList(StringType** arr) {
    int i = 0;
    printf("[");
    while (arr[i] != NULL) {
        if (i != 0) {
            printf(",");
        }
        printf("\"%s\"", arr[i]->buffer);
        i++;
    }
    printf("]");
}

int32_t lenStringList(StringType** arr) {
    int i = 0;
    while (arr[i] != NULL) {
        i++;
    }
    return i;
}

StringType** concatStringList(StringType** arrOne, StringType** arrTwo) {
    int sizeOne = lenStringList(arrOne);
    int sizeTwo = lenStringList(arrTwo);

    // add a NULL terminator
    StringType** stringArray = malloc((sizeOne + sizeTwo + 1) * sizeof(StringType *));
    stringArray[sizeOne + sizeTwo] = NULL;
    if (stringArray == NULL) {
        printf("Memory allocation failed\n");
        exit(1);
    }

    // Copy over first elements
    for (int i = 0; i < sizeOne; i++) {
        stringArray[i] = arrOne[i];
    }

    // Copy over second elements
    for (int i = 0; i < sizeTwo; i++) {
        stringArray[sizeOne + i] = arrTwo[i];
    }
    return stringArray;
}
