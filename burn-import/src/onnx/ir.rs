use half::f16;
use std::collections::HashMap;
use strum_macros::{Display, EnumString};

pub type Shape = Vec<usize>;

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: String,
    pub arg_type: Option<ArgType>,
}

#[derive(Debug, Clone)]
pub enum ArgType {
    Tensor(Tensor),
}

#[derive(Debug, Clone)]
pub struct SparseTensor(Tensor, Tensor, Shape);

#[derive(Debug, Clone)]
pub enum AttributeValue {
    Float32(f32),
    Int64(i64),
    String(String),
    Tensor(Tensor),
    // Graph(Graph),
    SparseTensor(SparseTensor),
    Float32s(Vec<f32>),
    Int64s(Vec<i64>),
    Strings(Vec<String>),
    Tensors(Vec<Tensor>),
    // Graphs(Vec<Graph>),
    SparseTensors(Vec<SparseTensor>),
}
pub type Attributes = HashMap<String, AttributeValue>;

// https://onnx.ai/onnx/intro/concepts.html#element-type

#[derive(Debug, Clone)]
pub enum ElementType {
    Float32,
    Float64,
    Int32,
    Int64,
    String,
    Float16,
    // Bfloat16,
    // Bool,
    // Complex128,
    // Complex64,
    // Int16,
    // Int8,
    // Uint16,
    // Uint32,
    // Uint64,
    // Uint8,
}

#[derive(Debug, Clone)]
pub struct Tensor {
    pub name: Option<String>,
    pub elem_type: ElementType,
    pub shape: Shape,
    pub data: Option<TensorData>,
}

#[derive(Debug, Clone)]
pub enum TensorData {
    Float16s(Vec<f16>),
    Float32s(Vec<f32>),
    Float64s(Vec<f64>),
    Int32s(Vec<i32>),
    Int64s(Vec<i64>),
    Strings(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub inputs: Vec<Argument>,
    pub outputs: Vec<Argument>,
    pub initializers: Vec<Argument>,
    pub old_node_names: HashMap<String, String>,
    pub old_input_names: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: NodeType,
    pub name: String,
    pub inputs: Vec<Argument>,
    pub outputs: Vec<Argument>,
    pub initializers: Vec<Argument>,
    pub attrs: Attributes,
    pub is_stateful: bool,
}

// Required by topological sort
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.node_type == other.node_type
    }
}

// Required by topological sort
impl Eq for Node {}

// Required by topological sort
impl core::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.node_type.hash(state);
        self.inputs.hash(state);
        self.outputs.hash(state);
    }
}

// Required by topological sort
impl core::hash::Hash for Argument {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// The list of supported node types (ONNX operators and some extra ones)
#[derive(Debug, Hash, Eq, PartialEq, EnumString, Clone, Display)]
pub enum NodeType {
    // Additional Burn operators
    Linear,
    Gelu,
    Conv1d,
    Conv2d,

    // ONNX operators
    Abs,
    Acos,
    Acosh,
    Add,
    And,
    ArgMax,
    ArgMin,
    Asin,
    Asinh,
    Atan,
    Atanh,
    AveragePool,
    BatchNormalization,
    Bernoulli,
    BitShift,
    BitwiseAnd,
    BitwiseNot,
    BitwiseOr,
    BitwiseXor,
    BlackmanWindow,
    Cast,
    CastLike,
    Ceil,
    Celu,
    CenterCropPad,
    Clip,
    Col,
    Im,
    Compress,
    Concat,
    ConcatFromSequence,
    Constant,
    ConstantOfShape,
    Conv,
    ConvInteger,
    ConvTranspose,
    Cos,
    Cosh,
    CumSum,
    DFT,
    DepthToSpace,
    DequantizeLinear,
    Det,
    Div,
    Dropout,
    DynamicQuantizeLinear,
    Einsum,
    Elu,
    Equal,
    Erf,
    Exp,
    Expand,
    EyeLike,
    Flatten,
    Floor,
    GRU,
    Gather,
    GatherElements,
    GatherND,
    Gemm,
    GlobalAveragePool,
    GlobalLpPool,
    GlobalMaxPool,
    Greater,
    GreaterOrEqual,
    GridSample,
    GroupNormalization,
    HammingWindow,
    HannWindow,
    HardSigmoid,
    HardSwish,
    Hardmax,
    Identity,
    If,
    InstanceNormalization,
    IsInf,
    IsNaN,
    LRN,
    LSTM,
    LayerNormalization,
    LeakyRelu,
    Less,
    LessOrEqual,
    Log,
    LogSoftmax,
    Loop,
    LpNormalization,
    LpPool,
    MatMul,
    MatMulInteger,
    Max,
    MaxPool,
    MaxRoiPool,
    MaxUnpool,
    Mean,
    MeanVarianceNormalization,
    MelWeightMatrix,
    Min,
    Mish,
    Mod,
    Mul,
    Multinomial,
    Neg,
    NegativeLogLikelihoodLoss,
    NonMaxSuppression,
    NonZero,
    Not,
    OneHot,
    Optional,
    OptionalGetElement,
    OptionalHasElement,
    Or,
    PRelu,
    Pad,
    Pow,
    QLinearConv,
    QLinearMatMul,
    QuantizeLinear,
    RNN,
    RandomNormal,
    RandomNormalLike,
    RandomUniform,
    RandomUniformLike,
    Range,
    Reciprocal,
    ReduceL,
    ReduceLogSum,
    ReduceLogSumExp,
    ReduceMax,
    ReduceMean,
    ReduceMin,
    ReduceProd,
    ReduceSum,
    ReduceSumSquare,
    Relu,
    Reshape,
    Resize,
    ReverseSequence,
    RoiAlign,
    Round,
    STFT,
    Scan,
    Scatter,
    ScatterElements,
    ScatterND,
    Selu,
    SequenceAt,
    SequenceConstruct,
    SequenceEmpty,
    SequenceErase,
    SequenceInsert,
    SequenceLength,
    SequenceMap,
    Shape,
    Shrink,
    Sigmoid,
    Sign,
    Sin,
    Sinh,
    Size,
    Slice,
    Softmax,
    SoftmaxCrossEntropyLoss,
    Softplus,
    Softsign,
    SpaceToDepth,
    Split,
    SplitToSequence,
    Sqrt,
    Squeeze,
    StringNormalizer,
    Sub,
    Sum,
    Tan,
    Tanh,
    TfIdfVectorizer,
    ThresholdedRelu,
    Tile,
    TopK,
    Transpose,
    Trilu,
    Unique,
    Unsqueeze,
    Upsample,
    Where,
    Xor,
}
