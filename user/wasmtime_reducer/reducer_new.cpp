#include <sys/time.h>
#include <iostream>
#include <vector>
#include <cstring>
#include <stdlib.h>
#include <cctype>
#include <fstream>
#include <sstream>
using namespace std;

#define MAX_SLOT_NUM 10
#define MAX_WORDS 18000000
#define MAX_BUFFER_SIZE 500000

__attribute__((import_module("env"), import_name("access_buffer"))) void access_buffer(void *slot_name, int name_size, void *buffer, int buffer_size);

void get_time() {
    timeval tv{};
    gettimeofday(&tv, nullptr);
    
    printf("%lld.%06ld\n", (long long)tv.tv_sec, tv.tv_usec);
}

string words[MAX_WORDS];
int counts[MAX_WORDS];

int main(int argc, char* argv[]) {
    int id = atoi(argv[1]);
    int mapper_num = atoi(argv[2]);
    cout << "reducer_" << id << " start!" << endl;
    int slot_num = mapper_num;
    string slot_name[MAX_SLOT_NUM];
    string buffer[MAX_SLOT_NUM];

    for (int i = 0; i < slot_num; ++i) {
        slot_name[i] = "buffer_" + to_string(i) + "_" + to_string(id);
        buffer[i] = string(MAX_BUFFER_SIZE, 0);
        access_buffer((void*)slot_name[i].c_str(), slot_name[i].length(), (void*)buffer[i].c_str(), buffer[i].size());
    }

    unordered_map<string, int> word_map;

    for (int i = 0; i < slot_num; i++) {
        istringstream iss(buffer[i]);
        string word;
        int num;
        while (iss >> word >> num) {
            word_map[word] += num;
        }
    }

    cout << "reducer_" << id << " solved " << word_map.size() << " words!" << endl;

    // string output_file = "faasm://reducer_" + to_string(id) + "_" + to_string(get_time()) + ".txt";
    // ofstream output(output_file);
    // for (int i = 0; i < word_index; i++) {
    //     output << words[i] << " " << count[i] << endl;
    // }
    // output.close();
    
    cout << "reducer_" << id << " finished!" << endl;
    get_time();
    return 0;
}