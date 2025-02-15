# easy_protobuf_edit

简单的Protobuf数据阅读和修改

simple Protobuf data reading and modification

## 使用 / Usage:

### 获取file_descriptor_set / Get file_descriptor_set

```shell
$ protoc --proto_path="." -o file_descriptor_set.bin example.proto
```

### 获取支持解析的类型全名 / Get fully qualified names of types that are supported for parsing

```shell
$ easy_protobuf_edit -d data.bin -f file_descriptor_set.bin
```

### 生成解析后的json文件 / Generate parsed json file

```shell
$ easy_protobuf_edit -d data.bin -f file_descriptor_set.bin -j data.json -m example.CustomMessage
```

### 修改生成后的Json文件 / Modify the generated Json file

### 根据Json文件生成Protobuf数据文件 / Generate Protobuf data files from Json files

```shell
$ easy_protobuf_edit -d data.bin -f file_descriptor_set.bin -j data.json -m example.CustomMessage -e
```

## 路线图 / Roadmap

- [x] Protobuf数据文件解析
- [x] 通过Json文件生成Protobuf数据文件
- [ ] 直接通过proto文件解析
- [ ] 通过Json文件输出Protobuf Base64编码数据
- [ ] 可视化网页
- [ ] 更多功能……