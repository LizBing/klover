//! Typed Java 8 attributes. Offsets are original bytecode offsets.
//!
//! Optional collections distinguish an absent attribute from a present empty
//! one. Debug tables retain separate occurrences in their original order.
use std::sync::Arc;

use super::constant_pool::{ClassRef, LoadableConstant, MethodHandleConstant};
use super::descriptor::FieldDescriptor;
use super::model::CodeData;
use super::text::JavaText;

#[derive(Debug)]
pub(crate) struct OpaqueAttribute {
    pub(crate) name: JavaText,
    /// Owned bytes; any embedded CP indices refer to ClassImage's original pool.
    pub(crate) body: Box<[u8]>,
}

#[derive(Debug, Default)]
pub(crate) struct AnnotationAttributes {
    pub(crate) visible: Option<Vec<Annotation>>,
    pub(crate) invisible: Option<Vec<Annotation>>,
    pub(crate) visible_types: Option<Vec<TypeAnnotation>>,
    pub(crate) invisible_types: Option<Vec<TypeAnnotation>>,
}

#[derive(Debug, Default)]
pub(crate) struct ClassAttributes {
    pub(crate) bootstrap_methods: Option<Vec<Arc<BootstrapMethod>>>,
    pub(crate) inner_classes: Option<Vec<InnerClass>>,
    pub(crate) enclosing_method: Option<EnclosingMethod>,
    pub(crate) signature: Option<JavaText>,
    pub(crate) source_file: Option<JavaText>,
    /// Modified UTF-8 debug text, preserved as bytes (no class-file Utf8 limit).
    pub(crate) source_debug_extension: Option<Box<[u8]>>,
    pub(crate) synthetic: bool,
    pub(crate) deprecated: bool,
    pub(crate) annotations: AnnotationAttributes,
    pub(crate) unknown: Vec<OpaqueAttribute>,
}

#[derive(Debug, Default)]
pub(crate) struct FieldAttributes {
    pub(crate) constant_value: Option<Arc<LoadableConstant>>,
    pub(crate) signature: Option<JavaText>,
    pub(crate) synthetic: bool,
    pub(crate) deprecated: bool,
    pub(crate) annotations: AnnotationAttributes,
    pub(crate) unknown: Vec<OpaqueAttribute>,
}

#[derive(Debug, Default)]
pub(crate) struct MethodAttributes {
    pub(crate) code: Option<CodeData>,
    pub(crate) exceptions: Option<Vec<Arc<ClassRef>>>,
    pub(crate) signature: Option<JavaText>,
    pub(crate) parameters: Option<Vec<MethodParameter>>,
    pub(crate) visible_parameter_annotations: Option<Vec<Vec<Annotation>>>,
    pub(crate) invisible_parameter_annotations: Option<Vec<Vec<Annotation>>>,
    pub(crate) annotation_default: Option<ElementValue>,
    pub(crate) synthetic: bool,
    pub(crate) deprecated: bool,
    pub(crate) annotations: AnnotationAttributes,
    pub(crate) unknown: Vec<OpaqueAttribute>,
}

#[derive(Debug, Default)]
pub(crate) struct CodeAttributes {
    pub(crate) stack_map_table: Option<Vec<StackMapFrame>>,
    pub(crate) line_number_tables: Vec<Vec<LineNumber>>,
    pub(crate) local_variable_tables: Vec<Vec<LocalVariable>>,
    pub(crate) local_variable_type_tables: Vec<Vec<LocalVariableType>>,
    pub(crate) visible_type_annotations: Option<Vec<TypeAnnotation>>,
    pub(crate) invisible_type_annotations: Option<Vec<TypeAnnotation>>,
    pub(crate) unknown: Vec<OpaqueAttribute>,
}

#[derive(Debug)]
pub(crate) struct BootstrapMethod {
    pub(crate) method: Arc<MethodHandleConstant>,
    pub(crate) arguments: Vec<Arc<LoadableConstant>>,
}

#[derive(Debug)]
pub(crate) struct InnerClass {
    pub(crate) inner: Arc<ClassRef>,
    pub(crate) outer: Option<Arc<ClassRef>>,
    pub(crate) inner_name: Option<JavaText>,
    pub(crate) access_flags: u16,
}

#[derive(Debug)]
pub(crate) struct EnclosingMethod {
    pub(crate) class: Arc<ClassRef>,
    pub(crate) method: Option<EnclosingMethodName>,
}

#[derive(Debug)]
pub(crate) struct EnclosingMethodName {
    pub(crate) name: JavaText,
    pub(crate) descriptor: super::descriptor::MethodDescriptor,
}

#[derive(Debug)]
pub(crate) struct MethodParameter {
    pub(crate) name: Option<JavaText>,
    pub(crate) access_flags: u16,
}

#[derive(Debug)]
pub(crate) struct Annotation {
    pub(crate) annotation_type: FieldDescriptor,
    pub(crate) elements: Vec<(JavaText, ElementValue)>,
}

#[derive(Debug)]
pub(crate) enum ElementValue {
    /// The original B/C/D/F/I/J/S/Z tag, and a constant of the required CP kind.
    Constant {
        tag: u8,
        value: Arc<LoadableConstant>,
    },
    /// Annotation strings refer directly to Utf8, not CONSTANT_String.
    String(JavaText),
    Enum {
        enum_type: FieldDescriptor,
        name: JavaText,
    },
    /// None represents the descriptor V (void.class).
    Class(Option<FieldDescriptor>),
    Annotation(Box<Annotation>),
    Array(Vec<ElementValue>),
}

#[derive(Debug)]
pub(crate) struct TypeAnnotation {
    pub(crate) target: TypeAnnotationTarget,
    pub(crate) path: Vec<TypePathStep>,
    pub(crate) annotation: Annotation,
}

#[derive(Debug)]
pub(crate) enum TypeAnnotationTarget {
    ClassTypeParameter(u8),
    MethodTypeParameter(u8),
    Supertype(u16),
    ClassTypeParameterBound {
        parameter: u8,
        bound: u8,
    },
    MethodTypeParameterBound {
        parameter: u8,
        bound: u8,
    },
    Field,
    MethodReturn,
    MethodReceiver,
    MethodParameter(u8),
    Throws(u16),
    LocalVariable(Vec<LocalVariableRange>),
    ResourceVariable(Vec<LocalVariableRange>),
    Catch(u16),
    /// target_type preserves which expression or type argument is annotated.
    Offset {
        target_type: u8,
        offset: u16,
    },
    TypeArgument {
        target_type: u8,
        offset: u16,
        argument: u8,
    },
}

#[derive(Debug)]
pub(crate) enum TypePathStep {
    ArrayElement,
    NestedType,
    WildcardBound,
    TypeArgument(u8),
}

#[derive(Debug)]
pub(crate) struct LocalVariableRange {
    pub(crate) start_pc: u16,
    pub(crate) length: u16,
    pub(crate) index: u16,
}

#[derive(Debug)]
pub(crate) struct StackMapFrame {
    /// Absolute offset, calculated from the encoded offset_delta.
    pub(crate) offset: u16,
    pub(crate) kind: StackMapFrameKind,
}

#[derive(Debug)]
pub(crate) enum StackMapFrameKind {
    Same,
    SameLocalsOneStack(VerificationType),
    Chop(u8),
    Append(Vec<VerificationType>),
    Full {
        locals: Vec<VerificationType>,
        stack: Vec<VerificationType>,
    },
}

#[derive(Debug)]
pub(crate) enum VerificationType {
    Top,
    Integer,
    Float,
    Double,
    Long,
    Null,
    UninitializedThis,
    Object(Arc<ClassRef>),
    Uninitialized(u16),
}

#[derive(Debug)]
pub(crate) struct LineNumber {
    pub(crate) start_pc: u16,
    pub(crate) line_number: u16,
}

#[derive(Debug)]
pub(crate) struct LocalVariable {
    pub(crate) range: LocalVariableRange,
    pub(crate) name: JavaText,
    pub(crate) descriptor: FieldDescriptor,
}

#[derive(Debug)]
pub(crate) struct LocalVariableType {
    pub(crate) range: LocalVariableRange,
    pub(crate) name: JavaText,
    pub(crate) signature: JavaText,
}
