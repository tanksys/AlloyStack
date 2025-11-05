#include <sys/time.h>
#include <stdio.h>
#include <iostream>
#include <vector>
#include <cstring>
#include <stdlib.h>
#include <cctype>
#include <fstream>
#include <algorithm>
#include <unordered_map>

using namespace std;

#define MAX_SLOT_NUM 10
#define MAX_WORDS 18000000
#define MAX_BUFFER_SIZE 500000

string words[MAX_WORDS];
int counts[MAX_WORDS];

__attribute__((import_module("env"), import_name("buffer_register"))) void buffer_register(void *slot_name, int name_size, void *buffer, int buffer_size);

void get_time() {
    timeval tv{};
    gettimeofday(&tv, nullptr);
    printf("%lld.%06lld\n", tv.tv_sec, tv.tv_usec);
}

int main(int argc, char* argv[])  {
    int id = atoi(argv[1]);
    int reducer_num = atoi(argv[2]);
    cout << "mapper_" << id << " start!" << endl;

    string input_file = "fake_data_" + to_string(id) + ".txt";
    ifstream file(input_file);

    string word;
    unordered_map<string, int> word_map;

    printf("start reading file\n");
    get_time();
    while (file >> word) {
        transform(word.begin(), word.end(), word.begin(),
                   [](unsigned char c) { return std::tolower(c); });
        ++word_map[word];
    }
    printf("finish reading file\n");
    get_time();
    // return 0;
    file.close();

    cout << "mapper_" << id << " read success!" << endl;

    string buffer[MAX_SLOT_NUM];
    string slot_name[MAX_SLOT_NUM];

    for (auto& pair : word_map) {
        int partition_index = hash<string>{}(pair.first) % reducer_num;
        slot_name[partition_index] = "buffer_" + to_string(partition_index) + "_" + to_string(id);
        buffer[partition_index].append(pair.first + " " + to_string(pair.second) + "\n");
    }

    for (int i = 0; i < reducer_num; i++) {
        buffer[i].resize(MAX_BUFFER_SIZE, '\0');
    }
    
    int word_index = word_map.size();
    cout << "mapper_" << id << " solved " << word_index << " words!" << endl;

    for (int i = 0; i < reducer_num; i++) {
        buffer_register((void*)slot_name[i].c_str(), slot_name[i].length(), (void*)buffer[i].c_str(), buffer[i].size());
        // cout << "buffersize" << i << ": " << buffer[i].size() << endl;
    }
    cout << "mapper_" << id << " write finished!" << endl;
    
    return 0;
}