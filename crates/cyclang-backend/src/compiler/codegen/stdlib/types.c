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
typedef struct Node Node;

typedef enum { INT32, INT64, FLOAT, STRING, LIST } Type;

typedef struct {
    Type type;
    union {
        int32_t i32;
        int64_t i64;
        float f;
        StringType *stringType;
        Node *listType;
    } data;
} Data;

typedef struct Node {
    Data data;
    struct Node *next;
} Node;

Node* createNode(Data data) {
    Node *newNode = (Node*)malloc(sizeof(Node));
    if (!newNode) {
        fprintf(stderr, "Memory allocation failed\n");
        exit(EXIT_FAILURE);
    }
    newNode->data = data;
    newNode->next = NULL;
    return newNode;
}

void push(Node **head, Data data) {
    Node *newNode = createNode(data);
    newNode->next = *head;
    *head = newNode;
}

Data pop(Node **head) {
    if (*head == NULL) {
        fprintf(stderr, "Attempt to pop from an empty list\n");
        exit(EXIT_FAILURE);  // or return a special Data value indicating failure
    }
    Node *temp = *head;
    Data poppedData = temp->data;
    *head = temp->next;
    free(temp); 
    return poppedData;
}

void printList(Node *head) {
    Node *current = head;
    printf("[");
    while (current != NULL) {
        switch (current->data.type) {
            case INT32:
                printf("%d", current->data.data.i32);
                break;
            case INT64:
                printf("%d", current->data.data.i32);
                break;
            case FLOAT:
                printf("%f", current->data.data.f);
                break;
            case STRING:
                stringPrint(current->data.data.stringType);
                break;
            case LIST:
                printList(current->data.data.listType);
                break;
        }
        current = current->next;
        if (current != NULL) {
            printf(",");
        }
    }
    printf("]");
}

void pushInt32(Node **head, int32_t value) {
    Data d1 = {.type = INT32, .data.i32 = value};
    push(head, d1);
}

void pushInt64(Node **head, int64_t value) {
    Data d1 = {.type = INT64, .data.i64 = value};
    push(head, d1);
}

void pushFloat(Node **head, float value) {
    Data d1 = {.type = FLOAT, .data.f = value};
    push(head, d1);
}

void pushString(Node **head, StringType *stringValue) {
    Data d1 = {.type = STRING, .data.stringType = stringValue};
    push(head, d1);
}

void pushList(Node **head, Node *list) {
    Data d1 = {.type = LIST, .data.listType = list};
    push(head, d1);
}

int test() {
    Node *head = NULL;
    Node *headTwo = NULL;
    Node *headThree = NULL;

    const char *data = "Hello, world!";
    StringType *myString = stringInit(data);


    pushInt32(&head, 5);
    pushInt32(&head, 10);
    pushInt32(&head, 15);

    pushInt32(&headTwo, -5);
    pushInt64(&headTwo, -10);
    pushInt32(&headTwo, -15);
    pushList(&headThree, head);
    pushList(&headThree, headTwo);
    // printList(headThree);
    pop(&headThree);
    pop(&headThree);
    printList(headThree);

    // Free the list
    while (head != NULL) {
        Node *temp = head;
        head = head->next;
        free(temp);
    }

    return 0;
 }

