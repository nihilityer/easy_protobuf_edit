# easy_protobuf_edit

简单的Protobuf数据阅读和修改

simple Protobuf data reading and modification

## 使用 / Usage:

### 获取file_descriptor_set / Get file_descriptor_set

```shell
$ protoc --proto_path="." -o file_descriptor_set.bin example.proto -g
```

### 1.1、直接使用GUI界面交互式编辑 / Directly use the GUI interface for interactive editing

```shell
$ easy_protobuf_edit
```

### 2.1、获取支持解析的类型全名 / Get fully qualified names of types that are supported for parsing

```shell
$ easy_protobuf_edit -d data.bin -f file_descriptor_set.bin -g
```

### 2.2、生成解析后的json文件 / Generate parsed json file

```shell
$ easy_protobuf_edit -d data.bin -f file_descriptor_set.bin -j data.json -m example.CustomMessage -g
```

### 2.3、修改生成后的Json文件 / Modify the generated Json file

### 2.4、根据Json文件生成Protobuf数据文件 / Generate Protobuf data files from Json files


```shell
$ easy_protobuf_edit -d data.bin -f file_descriptor_set.bin -j data.json -m example.CustomMessage -e -g
```

## 路线图 / Roadmap

- [x] Protobuf数据文件解析
- [x] 通过Json文件生成Protobuf数据文件
- [ ] 更好用的页面交互
- [ ] 通过Json文件输出Protobuf Base64编码数据
- [ ] 可视化网页
- [ ] 更多功能……